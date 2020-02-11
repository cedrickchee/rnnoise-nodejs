const rnnoise = require("../.");

const denoisedBufLength = rnnoise.suppress(
  "/home/cedric/Downloads/temp_01/babble_15dB.wav",
  "/home/cedric/Downloads/temp_01/babble_15dB_dn.wav"
);

console.log(`Denoised buffer length: ${denoisedBufLength} bytes`);
