function beforeUploadFormRequest() {
  if (document.getElementById("upload_form").innerHTML) {
    event.preventDefault();
    document.getElementById("upload_form").style.display = "block";
  }
}
