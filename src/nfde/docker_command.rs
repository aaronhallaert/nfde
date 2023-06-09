use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs::read, io::Cursor, process::Stdio};

use anyhow::bail;
use lib::config::Config;
use skim::{
    prelude::{SkimItemReader, SkimOptionsBuilder},
    Skim,
};

use crate::{DockerAction, DockerCommand};

fn config() -> Config {
    lib::config::get_config().unwrap()
}

pub fn handle_docker_command(docker_command: DockerCommand) -> anyhow::Result<()> {
    match docker_command.docker_action {
        DockerAction::Save(docker_save_command) => {
            save(docker_save_command.name)?;
        }
        DockerAction::Load(docker_load_command) => {
            load(docker_load_command.name)?;
        }
        DockerAction::Remove(docker_remove_command) => {
            remove(docker_remove_command.name)?;
        }
    };

    Ok(())
}

fn save(name: Option<String>) -> anyhow::Result<()> {
    match name {
        Some(name) => {
            let image_path = Path::new(&config().image_folder()).join(format!("{}.tar", name));

            println!("Saving docker image to {}", &image_path.display());

            let ran = {
                let mut cmd = ::std::process::Command::new("docker");
                cmd.arg("save");
                cmd.arg("-o");
                cmd.arg(image_path);
                cmd.arg(&config().api_image_name);
                cmd
            }
            .status()
            .unwrap()
            .success();

            if ran {
                println!("Successfully saved docker image");

                Ok(())
            } else {
                Err(anyhow::anyhow!("Could not save docker image"))
            }
        }
        None => {
            bail!("Please provide a name for the image")
        }
    }
}

fn load(name: Option<String>) -> anyhow::Result<()> {
    let image_path = determine_image_path(name)?;

    let image_data = read(&image_path).expect("Failed to read image file");

    println!("Loading docker image {}", &image_path.display());

    let mut child = ::std::process::Command::new("docker")
        .arg("load")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to spawn docker command");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&image_data)
            .expect("Failed to write image data to stdin");
        stdin.flush().expect("Failed to flush stdin");
    }

    let ran = child
        .wait()
        .expect("Failed to wait for docker command")
        .success();

    if ran {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Could not load docker image"))
    }
}

fn remove(name: Option<String>) -> anyhow::Result<()> {
    let image_path = determine_image_path(name)?;

    let ran = {
        let mut cmd = ::std::process::Command::new("rm");
        cmd.arg(&image_path);
        cmd
    }
    .status()
    .unwrap()
    .success();

    if ran {
        println!("Removed image: {}", &image_path.display());
        Ok(())
    } else {
        Err(anyhow::anyhow!("Could not remove docker image"))
    }
}

fn determine_image_path(name: Option<String>) -> anyhow::Result<PathBuf> {
    let image_path = match name {
        Some(name) => Path::new(&config().image_folder()).join(format!("{}.tar", name)),
        None => {
            let selected_file = select_image();
            match selected_file {
                Ok(file) => {
                    Path::new(&config().image_folder()).join(file)
                }
                Err(e) => bail!(e),
            }
        }
    };

    // check if file exists
    if !std::path::Path::new(&image_path).exists() {
        bail!("File does not exist: {}", &image_path.display());
    }

    //check if file extension is sql
    if !&image_path.display().to_string().ends_with(".tar") {
        bail!("File is not a tar file: {}", &image_path.display());
    }

    Ok(image_path)
}

fn select_image() -> anyhow::Result<String> {
    let options = SkimOptionsBuilder::default()
        .height(Some("100%"))
        .multi(true)
        .build()
        .unwrap();

    let files_in_folder = std::fs::read_dir(config().image_folder()).unwrap();

    let joined_by_newline = files_in_folder
        .filter(|file| {
            file.as_ref()
                .unwrap()
                .file_name()
                .into_string()
                .unwrap()
                .ends_with(".tar")
        })
        .map(|file| file.unwrap().file_name().into_string().unwrap())
        .collect::<Vec<String>>()
        .join("\n");

    let item_reader = SkimItemReader::default();

    let items = item_reader.of_bufread(Cursor::new(joined_by_newline));

    let skim_output = Skim::run_with(&options, Some(items))
        .ok_or_else(|| anyhow::anyhow!("Skim failed"))
        .unwrap();

    if skim_output.is_abort {
        return Err(anyhow::anyhow!("Image selection aborted"));
    }

    let selected_filename = skim_output
        .selected_items
        .get(0)
        .unwrap()
        .output()
        .to_string();

    Ok(selected_filename)
}
