import init, { detect, Detector } from "./pack_stack.js";

const video = document.getElementById("video");
const output = document.getElementById("output");
const context = output.getContext("2d");

let detector;

const structLoadData = () => {
  console.log(video.clientHeight, video.clientWidth);
  output.width = video.clientWidth;
  output.height = video.clientHeight;

  detector = Detector.new(video.clientWidth, video.clientHeight);

  window.requestAnimationFrame(structTick);
};

const functionLoadData = () => {
  console.log(video.clientHeight, video.clientWidth);
  output.width = video.clientWidth;
  output.height = video.clientHeight;

  window.requestAnimationFrame(functionTick);
};

video.addEventListener("loadeddata", structLoadData, false);

const structTick = () => {
  context.drawImage(video, 0, 0, video.clientWidth, video.clientHeight);
  const imageData = context.getImageData(0, 0, video.clientWidth, video.clientHeight);
  const data = detector.detect(imageData.data);
  context.putImageData(new ImageData(data, video.clientWidth, video.clientHeight), 0, 0);
  window.requestAnimationFrame(structTick);
};

const functionTick = () => {
  context.drawImage(video, 0, 0, video.clientWidth, video.clientHeight);
  const imageData = context.getImageData(0, 0, video.clientWidth, video.clientHeight);
  const data = detect(imageData.data, video.clientWidth, video.clientHeight);
  context.putImageData(new ImageData(data, video.clientWidth, video.clientHeight), 0, 0);
  window.requestAnimationFrame(functionTick);
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
  });

})();
