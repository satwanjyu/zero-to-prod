mod health_check;
pub use health_check::health_check;
mod home;
pub use home::home;
mod login;
pub use login::{login, login_form};
mod subscriptions;
pub use subscriptions::subscribe;
mod subscriptions_confirm;
pub use subscriptions_confirm::confirm;
mod admin;
pub use admin::{
    admin_dashboard, change_password, change_password_form, logout, publish_newsletter,
    publish_newsletter_form,
};
