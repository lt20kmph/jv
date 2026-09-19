// Global Variables

var c = null;

var modifiedFile = null;

// Functions

// Croppie writes its boundary size as inline px, so a hardcoded 450 overflowed
// the screen horizontally on any phone narrower than that. Size it to whatever
// the form column actually offers, capped at the original 450.
const croppieSize = () => {
  const host = document.getElementById("croppie");
  const available = host?.parentElement?.clientWidth || window.innerWidth;
  return Math.max(200, Math.min(450, Math.floor(available - 24)));
};

const croppie = () => {
  if (c == null) {
    const size = croppieSize();
    c = new Croppie(document.getElementById("croppie"), {
      enableExif: true,
      viewport: { width: size, height: size },
      boundary: { width: size, height: size },
      showZoomer: false,
      enableResize: false,
      enableOrientation: false,
      enforceBoundary: true,
      customClass: "croppie-container",
      mouseWheelZoom: true,
    });
  }
  return c;
};

const uploadFile = () => {
  const img = event.target.files[0];
  const url = URL.createObjectURL(img);
  const size = croppieSize();
  croppie()
    .bind({
      url: url,
      points: [0, 0, size, size],
    })
    .then(() => {
      c.setZoom(0);
    });
};

async function setModifiedFile() {
  if (modifiedFile == null) {
    const blob = await croppie().result({
      type: "blob",
      format: "jpeg",
      quality: 1,
      size: { width: 900 },
    });
    modifiedFile = new File([blob], Date.now().toString(), {
      type: "image/jpeg",
    });
  }
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
