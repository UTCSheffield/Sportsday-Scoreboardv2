import { Controller } from "@hotwired/stimulus";

export default class extends Controller {
  static override values = {
    key: String,
    value: String,
  };

  filter() {
    const params = new URLSearchParams(location.search);
    if (this.valueValue != "all") {
      params.set(this.keyValue, this.valueValue);
    } else {
      params.delete(this.keyValue);
    }
    console.log("Dispatching Safe Score Redirect");
    document.dispatchEvent(
      new CustomEvent("doSafeScoreRedirect", {
        detail: {
          params: `?${params}`,
        },
      }),
    );
  }

  declare keyValue: string;
  declare hasKeyValue: boolean;
  declare valueValue: string;
  declare hasValueValue: boolean;
}
