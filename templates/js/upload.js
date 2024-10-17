// Global Variables

var c = null;

if (c == null) {
  c = new Croppie(document.getElementById("croppie"), {
    enableExif: true,
    viewport: { width: 100, height: 100 },
    boundary: { width: 300, height: 300 },
    showZoomer: false,
    enableResize: true,
    enableOrientation: true,
    mouseWheelZoom: "ctrl",
  });
}

var modifiedFile = null;

// Functions

async function setModifiedFile() {
  if (modifiedFile == null) {
    const blob = await c.result({
      type: "blob",
      format: "jpeg",
      quality: 1,
    });
    modifiedFile = new File([blob], Date.now().toString(), {
      type: "image/jpeg",
    });
  }
}

function uploadFile(file) {
  const img = event.target.files[0];
  const url = URL.createObjectURL(img);
  c.bind({
    url: url,
    points: [77, 469, 280, 739],
  });
}

// Event Listeners

document.body.addEventListener("htmx:confirm", (evt) => {
  if (modifiedFile == null) {
    evt.preventDefault();
    setModifiedFile().then(() => evt.detail.issueRequest());
  }
});

document.body.addEventListener("htmx:configRequest", (evt) => {
  evt.detail.parameters["modified_file"] = modifiedFile;
  modifiedFile = null;
});
