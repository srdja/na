
pub fn render_ui(res: &Vec<String>) -> String {
    let upload =
"<!DOCTYPE html>
<html>
<head>
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

    for r in res {
        files = files +
            &(format!("  <a href=\"{}\" target=\"_blank\">{}</a></br>\n", r, r));
    }
    let page = upload + &files + "</body>\n</html>";
    page
}
