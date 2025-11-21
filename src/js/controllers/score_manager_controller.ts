import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
    static override values = {
        lock: Boolean
    }
    static override targets = ["container"]

    override connect() {
        if (this.getLockState()) {
            console.log("Failed to obtain lock, exiting");
            return
        }

        document.addEventListener("scoreUpdate", (e: any) => {
            if (!this.getLockState()) {
                this.lockValue = String(true);
                console.log("Score Manager: Obtained Lock")
                const data = JSON.parse(this.containerTarget.innerText);
                data[e.detail.event_id] = e.detail.scores;
                this.containerTarget.innerText = JSON.stringify(data);
                this.lockValue = String(false);
                console.log("Score Manager: Released Lock")
            }
        })

        document.addEventListener("doSafeScoreRedirect", async (e: any) => {
            if (!this.getLockState()) {
                this.lockValue = String(true);
                console.log("Score Manager: Obtained Lock")
                const data = JSON.parse(this.containerTarget.innerText);
                if (Object.keys(data).length != 0) {
                    let res = await fetch("/set_scores", {
                        method: "POST",
                        body: this.containerTarget.innerText,
                    });
                    if (res.status == 204) {
                        this.containerTarget.innerText = "{}";
                    } else {
                        console.log(`Set Scores failed with status ${res.status}:`, res.body);
                    }
                }
                location.search = e.detail.params
                this.lockValue = String(false);
                console.log("Score Manager: Released Lock")
            }
        })

        setInterval(async () => {
            if (!this.getLockState()) {
                this.lockValue = String(true);
                console.log("Score Manager: Obtained Lock");
                let res = await fetch("/set_scores", {
                    method: "POST",
                    body: this.containerTarget.innerText,
                });
                if (res.status == 204) {
                    this.containerTarget.innerText = "{}";
                } else {
                    console.log(`Set Scores failed with status ${res.status}:`, res.body);
                }
                this.lockValue = String(false);
                console.log("Score Manager: Released Lock")
            } else {
                console.log("Data Locked, Will try again in 5s");
            }
        }, 5000)
    }

    getLockState() {
        return Boolean(this.lockValue);
    }

    declare lockValue: string;
    declare hasLockValue: boolean;
    declare containerTarget: HTMLElement;
}