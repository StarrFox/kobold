use std::{
    env,
    path::{Path, PathBuf},
};

use katsuba_executor::{Buffer, Executor, Task};
use katsuba_wad::{Archive, Inflater};

use crate::{cli::OutputSource, utils::DirectoryTree};

struct SafeArchiveDrop<'a> {
    ex: &'a Executor,
    archive: Archive,
}

impl Drop for SafeArchiveDrop<'_> {
    fn drop(&mut self) {
        // Join all pending tasks on the executor to make sure none of
        // them hold onto dangling `archive` references anymore after
        // dropping it.
        self.ex.join().for_each(drop);
    }
}

fn fetch_file_contents<'a>(
    ex: &'a Executor,
    archive: &'a Archive,
    inflater: &mut Inflater,
    file: &katsuba_wad::types::File,
) -> eyre::Result<Option<Buffer<'a>>> {
    if file.is_unpatched {
        return Ok(None);
    }

    let contents = archive
        .file_contents(file)
        .ok_or_else(|| eyre::eyre!("missing file contents in archive"))?;

    match file.compressed {
        true => {
            let len = file.uncompressed_size as usize;

            ex.request_buffer(len, |buf| {
                buf.resize(len, 0);
                inflater.decompress_into(buf, contents)?;

                Ok(())
            })
            .map(Some)
        }

        false => Ok(Some(Buffer::borrowed(contents))),
    }
}

fn create_directory_tree(ex: &Executor, archive: &Archive, out: &Path) -> eyre::Result<()> {
    // Pre-compute the directory structure we need to create.
    let mut tree = DirectoryTree::new();
    for file in archive.files().keys() {
        tree.add(file.as_ref());
    }

    // Create all the directories with minimal required syscalls.
    for path in tree {
        let task = Task::create_dir(out.join(path));
        for pending in ex.dispatch(task) {
            pending?;
        }
    }

    // Join all pending operations here so we don't accidentally
    // try to write into directories that don't exist yet.
    for pending in ex.join() {
        pending?;
    }

    Ok(())
}

pub fn extract_archive(
    ex: &Executor,
    inpath: Option<PathBuf>,
    archive: Archive,
    out: OutputSource,
) -> eyre::Result<()> {
    // Determine the output directory for the archive files.
    // Since we can't print here, we use the cwd instead.
    let input_stem = inpath.as_ref().and_then(|p| p.file_stem()).unwrap();
    let mut out = match out {
        OutputSource::Stdout => env::current_dir()?,
        OutputSource::File(p) | OutputSource::Dir(p, ..) => p,
    };
    out.push(input_stem);

    // First, create all the directories for the output files.
    create_directory_tree(ex, &archive, &out)?;

    // This guard ensures we can safely share references into `archive`
    // with the pool without risking dangling in the case of an error.
    let sad = SafeArchiveDrop { ex, archive };
    let mode = sad.archive.mode();

    // Next, we do the extraction of data out of the archive on the
    // current thread while simultaneously dispatching the file I/O
    // operations to the executor.
    let mut inflater = Inflater::new();
    for (path, file) in sad.archive.files() {
        let path = out.join(path);

        // SAFETY: We can never end up with dangling references into
        // `archive` because `sad` joins all pending tasks on drop.
        let buffer = match fetch_file_contents(ex, &sad.archive, &mut inflater, file)? {
            Some(buf) => buf,
            None => {
                log::warn!("Skipping unpatched file '{}'", path.display());
                continue;
            }
        };
        let buffer = unsafe { buffer.extend_lifetime() };

        let task = Task::create_file(path, buffer, mode);
        for pending in ex.dispatch(task) {
            pending?;
        }
    }

    Ok(())
}
