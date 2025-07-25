use colored::*;
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Animated spinner with color changes
pub struct Spinner {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    message: String,
}

impl Spinner {
    /// Create a new spinner with a message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
            message: message.into(),
        }
    }

    /// Start the spinner animation
    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            return; // Already running
        }

        self.running.store(true, Ordering::Relaxed);
        let running = self.running.clone();
        let message = self.message.clone();

        let handle = thread::spawn(move || {
            let frames = ['|', '/', '-', '\\'];
            let colors = [Color::Red, Color::Yellow, Color::Green, Color::Cyan];
            let mut frame_index = 0;

            // Hide cursor
            print!("\x1B[?25l");
            io::stdout().flush().unwrap();

            while running.load(Ordering::Relaxed) {
                let frame = frames[frame_index % frames.len()];
                let color = colors[frame_index % colors.len()];

                // Move to beginning of line, clear it, and print spinner
                print!("\r{} {}", frame.to_string().color(color), message);
                io::stdout().flush().unwrap();

                frame_index += 1;
                thread::sleep(Duration::from_millis(100));
            }

            // Clear the line and show cursor again
            print!("\r\x1B[K\x1B[?25h");
            io::stdout().flush().unwrap();
        });

        self.handle = Some(handle);
    }

    /// Stop the spinner animation
    pub fn stop(&mut self) {
        if !self.running.load(Ordering::Relaxed) {
            return; // Not running
        }

        self.running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
    }

    /// Stop the spinner and print a success message
    pub fn success(&mut self, message: impl Into<String>) {
        self.stop();
        println!("{} {}", "[SUCCESS]".green(), message.into());
    }

    /// Stop the spinner and print an error message
    pub fn error(&mut self, message: impl Into<String>) {
        self.stop();
        println!("{} {}", "[ERROR]".red(), message.into());
    }

    /// Stop the spinner and print an info message
    pub fn info(&mut self, message: impl Into<String>) {
        self.stop();
        println!("{} {}", "[INFO]".cyan(), message.into());
    }

    /// Update the spinner message while it's running
    pub fn update_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Convenience function to run a future with a spinner
pub async fn with_spinner<F, T>(message: impl Into<String>, future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let mut spinner = Spinner::new(message);
    spinner.start();
    let result = future.await;
    spinner.stop();
    result
}

/// Convenience function to run a closure with a spinner
pub fn with_spinner_sync<F, T>(message: impl Into<String>, closure: F) -> T
where
    F: FnOnce() -> T,
{
    let mut spinner = Spinner::new(message);
    spinner.start();
    let result = closure();
    spinner.stop();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_spinner_basic() {
        let mut spinner = Spinner::new("Testing...");
        spinner.start();
        thread::sleep(Duration::from_millis(500));
        spinner.stop();
    }

    #[test]
    fn test_spinner_success() {
        let mut spinner = Spinner::new("Processing...");
        spinner.start();
        thread::sleep(Duration::from_millis(200));
        spinner.success("Operation completed!");
    }

    #[tokio::test]
    async fn test_with_spinner() {
        let result = with_spinner("Loading data...", async {
            tokio::time::sleep(Duration::from_millis(100)).await;
            "test result"
        })
        .await;

        assert_eq!(result, "test result");
    }
}
