import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static override values = {
    score: String,
  };

  static override targets = ["select"];

  override connect() {
    this.selectTargets.forEach((selectElement) => {
      try {
        const eventScores = JSON.parse(this.scoreValue);
        const formId = selectElement.getAttribute("data-form-id");

        if (!formId) {
          selectElement.value = "0";
          return;
        }

        const formScore = eventScores[formId];

        if (formScore !== undefined && formScore !== null) {
          // Set the select value to the stored score
          selectElement.value = formScore.toString();
        } else {
          // Fallback to "Nothing" (value 0)
          selectElement.value = "0";
        }
      } catch (e) {
        // If JSON parsing fails or scores is empty, fallback to "Nothing"
        selectElement.value = "0";
      }
    });
  }

  declare scoreValue: string;
  declare readonly hasScoreValue: boolean;

  declare readonly hasSelectTarget: boolean;
  declare readonly selectTargets: HTMLSelectElement[];
}
