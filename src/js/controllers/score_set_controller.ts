import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static override targets = ["field", "form", "button"];
  static override values = {
    status: String,
  };

  override connect() {
    document.addEventListener("scoreUpdateSubmitted", (e: any) => {
      if (this.statusValue == "pending") {
        console.log("Setting Button Target to Green");
        this.buttonTarget.style.backgroundColor = "green";
        this.statusValue = "completed";
      }
    });
  }

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

    if (this.hasButtonTarget) {
      console.log("Setting Button Target to Yellow");
      this.buttonTarget.style.backgroundColor = "yellow";
      this.statusValue = "pending";
    }
  }

  declare readonly hasFieldTarget: boolean;
  declare readonly fieldTarget: HTMLInputElement;
  declare readonly fieldTargets: HTMLSelectElement[];

  declare readonly hasFormTarget: boolean;
  declare readonly formTarget: HTMLInputElement;
  declare readonly formTargets: HTMLSelectElement[];

  declare readonly hasButtonTarget: boolean;
  declare readonly buttonTarget: HTMLInputElement;
  declare readonly buttonTargets: HTMLButtonElement[];

  declare statusValue: string;
  declare readonly hasStatusValue: boolean;
}
