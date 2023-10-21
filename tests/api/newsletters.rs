use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{assert_is_redirected_to, spawn_app, ConfirmationLinks, TestApp};

#[tokio::test]
async fn user_must_be_logged_in_to_see_newletters_form() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get_newsletters().await;

    // Assert
    assert_is_redirected_to(&response, "/login");
}

#[tokio::test]
async fn user_must_be_logged_in_to_send_newsletters() {
    // Arrange
    let app = spawn_app().await;

    // Act - Part 1
    create_confirmed_subscriber(&app).await;

    // Act - Part 2 - Post newsletter
    let response = app
        .post_newsletters(serde_json::json!({
            "title": "Newsletter title",
            "html_content": "<p>Newsletter body as HTML</p>",
            "text_content": "Newsletter body as plain text."
        }))
        .await;

    // Assert
    assert_is_redirected_to(&response, "/login");
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    let email_server = Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0);

    // Act - Part 1
    create_unconfirmed_subscriber(&app).await;

    // Act - Part 2 - Login
    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;

    // Act - Part 3 - Post newsletter
    email_server.mount(&app.email_server).await;
    let response = app
        .post_newsletters(serde_json::json!({
            "title": "Newsletter title",
            "html_content": "<p>Newsletter body as HTML</p>",
            "text_content": "Newsletter body as plain text."
        }))
        .await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    // Mock verifies receiving no requests
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber.")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions("name=le%20guin&email=ursula_le_guin%40gmail.com".into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_links(email_request)
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;

    let email_server = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1);

    // Act - Part 1
    create_confirmed_subscriber(&app).await;

    // Act - Part 2 - Login
    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;

    // Act - Part 3 - Post newsletter
    email_server.mount(&app.email_server).await;
    let response = app
        .post_newsletters(serde_json::json!({
            "title": "Newsletter title",
            "html_content": "<p>Newsletter body as HTML</p>",
            "text_content": "Newsletter body as plain text."
        }))
        .await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    // Mock verifies receiving one request
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_link.html)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletters_returns_400_for_invalid_body() {
    // Arrange
    let app = spawn_app().await;

    // Act - Part 1
    create_confirmed_subscriber(&app).await;

    // Act - Part 2 - Login
    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;

    // Act - Part 3 - Post newsletter
    let test_cases = [
        (
            serde_json::json!({
                    "html_content": "<p>Newsletter body as HTML</p>",
                    "text_content": "Newsletter body as plain text"
            }),
            "missing title",
        ),
        (
            serde_json::json!({"title": "Newsletter!"}),
            "missing content",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = app.post_newsletters(invalid_body).await;

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
