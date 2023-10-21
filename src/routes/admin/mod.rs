mod dashboard;
pub use dashboard::admin_dashboard;
mod password;
pub use password::{change_password, change_password_form};
mod logout;
pub use logout::logout;
mod newsletters;
pub use newsletters::{publish_newsletter, submit_newsletter_form};
