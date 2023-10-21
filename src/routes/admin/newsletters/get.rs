use actix_web::HttpResponse;
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn submit_newsletter_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_html = String::new();
    for m in flash_messages.iter() {
        writeln!(error_html, "<p><i>{}</i></p>", m.content()).unwrap();
    }

    HttpResponse::Ok().body(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html, charset=utf-8">
    <title>Submit Newsletter</title>
</head>
<body>
    {error_html}
    <form action="admin/newsletters" method="post">
        <label>Title
            <input
                type="text"
                placeholder="Enter Title"
                name="title"
            >
        </label>
        <br>
        <label>HTML Content
            <input
                type="text"
                placeholder="Enter Content in HTML"
                name="html_content"
            >
        </label>
        <br>
        <label>Text Content
            <input
                type="text"
                placeholder="Enter Content in Plain Text"
                name="text_content"
            >
        </label>
        <br>
        <button type="submit">Submit</button>
    </form>
    <p><a href="/admin/dashboard">&lt;- Back</a></p>
</body>
</html>"#,
    ))
}
