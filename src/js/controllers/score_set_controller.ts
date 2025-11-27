import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static override targets = ["field", "form"];

  override connect() {}

  submit() {
    console.log("Submitting a score to the manager");
    const obj: Record<string, string> = {};
    this.fieldTargets.forEach((val) => {
      let form_id = val.id.split("-")[3];
      obj[form_id!] = val.value;
    });

    document.dispatchEvent(
      new CustomEvent("scoreUpdate", {
        detail: { event_id: this.formTarget.id, scores: obj },
      }),
    );
  }

  declare readonly hasFieldTarget: boolean;
  declare readonly fieldTarget: HTMLInputElement;
  declare readonly fieldTargets: HTMLSelectElement[];

  declare readonly hasFormTarget: boolean;
  declare readonly formTarget: HTMLInputElement;
  declare readonly formTargets: HTMLSelectElement[];
}
