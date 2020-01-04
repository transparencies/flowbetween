use super::output::*;
use super::command::*;

use flo_stream::*;
use futures::prelude::*;
use futures::stream;
use futures::task::{Poll};

///
/// Runs a series of commands provided by a stream and returns a stream of the resulting output
///
pub fn flo_run_commands<InputStream>(commands: InputStream) -> impl Stream<Item=FloCommandOutput>+Unpin
where InputStream: 'static+Stream<Item=FloCommand>+Unpin+Send {
    // Create the output
    let mut output_publisher    = Publisher::new(1);
    let mut output              = output_publisher.subscribe();
    let mut runner              = Some(run_commands(commands, output_publisher).boxed_local());

    // Reading from the output stream causes commands to be run
    stream::poll_fn(move |context| {
        // Try to run a command
        if let Some(ref mut active_runner) = runner {
            if active_runner.poll_unpin(context) == Poll::Ready(()) {
                // Command has completed: free up the runner
                runner = None;
            }
        }

        // Try to read some output. We stop running when the output stream is no longer being read from
        output.poll_next_unpin(context)
    })
}

///
/// Runs the specified series of commands and writes the output to the given publisher
///
async fn run_commands<InputStream>(mut commands: InputStream, mut output: Publisher<FloCommandOutput>)
where InputStream: 'static+Stream<Item=FloCommand>+Unpin {
    while let Some(command) = commands.next().await {
        // Commands begin and end with a 'begin/finish' output
        output.publish(FloCommandOutput::BeginCommand(command.clone()));

        match command {
            FloCommand::Version     =>  { output.publish(FloCommandOutput::Message(format!("{} ({}) v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_DESCRIPTION"), env!("CARGO_PKG_VERSION")))).await; }
        }

        // Finish the command
        output.publish(FloCommandOutput::FinishCommand(command.clone()));
    }
}
