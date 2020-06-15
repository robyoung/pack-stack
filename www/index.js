import init, { Detector } from "./pack_stack.js";

const video = document.getElementById("video");
const output = document.getElementById("output");
const context = output.getContext("2d");

let detector;

const loadData = () => {
  console.log(video.videoHeight, video.videoWidth);
  output.width = video.videoWidth;
  output.height = video.videoHeight;

  detector = Detector.new(video.videoWidth, video.videoHeight, true);

  window.requestAnimationFrame(tick);
};

video.addEventListener("loadeddata", loadData, false);

const tick = () => {
  context.drawImage(video, 0, 0, video.videoWidth, video.videoHeight);
  const imageData = context.getImageData(0, 0, video.videoWidth, video.videoHeight);
  const data = detector.detect(imageData.data);
  if (detector.boundary_match()) {
    console.log("CAPTURE");
    let capture = document.createElement("canvas");
    capture.width = output.width;
    capture.height = output.height;
    let captureCtx = capture.getContext("2d");
    captureCtx.putImageData(imageData.data, 0, 0);
    document.getElementById("captures").appendChild(capture);
  }
  context.putImageData(new ImageData(data, video.videoWidth, video.videoHeight), 0, 0);
  window.requestAnimationFrame(tick);
};

(async () => {
  await init();

  // setup video
  const videoConstraints = {
      audio: false,
      video: { 
        facingMode: "environment",
      },
  };
  navigator.mediaDevices.getUserMedia(videoConstraints).then(stream => {
    video.srcObject = stream;
    video.play();
  });

})();
