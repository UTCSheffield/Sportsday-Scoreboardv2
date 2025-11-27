import { bunStimulusPlugin } from "bun-stimulus-plugin";

await Bun.build({
  entrypoints: ["src/js/index.ts"],
  outdir: "assets",
  minify: true,
  sourcemap: "none",
  env: "PUBLIC_*",
  plugins: [bunStimulusPlugin()],
});
