const rnnoise = require("../.");

const denoisedBufLength = rnnoise.suppress(
  "babble_15dB.wav",
  "babble_15dB_dn.wav"
);

console.log(`Denoised buffer length: ${denoisedBufLength} bytes`);
