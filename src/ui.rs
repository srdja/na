
use std::collections::HashMap;


pub fn render_ui(res: &HashMap<String, String>) -> String {
    let upload =
"<!DOCTYPE html>
<html>
<head>
  <meta charset=\"utf-8\">
  <title>Sharing is caring</title>
</head>
<body>

  <form action=\"\" method=\"post\" enctype=\"multipart/form-data\">
  <input type=\"file\" name=\"upload\" id=\"filename\"></br></br>
  <input type=\"submit\" value=\"upload\">
  </form>

  <hr>
  Files:</br></br>\n".to_string();

    let mut files = String::new();

    for (uri, name) in res {
        files = files +
            &(format!("  <a href=\"{}\" target=\"_blank\">{}</a></br>\n", uri, name));
    }
    let page = upload + &files + "</body>\n</html>";
    page
}
