use super::args::{SampleArgs, SampleName};
use super::error::Result;
use super::output::{Streams, report_new_folder, write_atomic};

pub(crate) fn sample(a: &SampleArgs, io: &mut Streams) -> Result {
    let d = match a.name {
        SampleName::Bracket => forja_engine::sample::bracket(),
        SampleName::Floorplan => forja_engine::sample::floor_plan(),
    };
    let bytes = forja_io::write(&d, &a.output)?;
    let folder = write_atomic(&a.output, &bytes)?;
    report_new_folder(folder, io);
    io.errln(format_args!("wrote {} ({} bytes)", a.output.display(), bytes.len()));
    Ok(())
}
