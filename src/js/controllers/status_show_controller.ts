import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  override connect() {
    console.log("Connected Status Update Controller");
    document.addEventListener("updateStatus", (e: any) => {
      this.element.innerHTML = e.detail.status;
    });
  }
}
