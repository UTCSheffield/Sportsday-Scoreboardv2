import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static override targets = ["output"];

  override connect() {
    document.addEventListener("wsmessage", (e: any) => {
      if (e.detail.channel != "scores") return;
      this.outputTarget.innerHTML = e.detail.data;
      console.log("Updated Scores");
    });
  }

  declare readonly hasOutputTarget: boolean;
  declare readonly outputTarget: HTMLDivElement;
}
