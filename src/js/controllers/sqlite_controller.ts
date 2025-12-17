import { Controller } from "@hotwired/stimulus";

interface SqliteResult {
  success: boolean;
  output: string;
  error?: string;
}

export default class extends Controller<HTMLElement> {
  static override targets = ["terminal", "history", "input"];

  declare readonly terminalTarget: HTMLElement;
  declare readonly historyTarget: HTMLElement;
  declare readonly inputTarget: HTMLInputElement;

  private commandHistory: string[] = [];

  override connect() {
    // Focus the input when the controller connects
    this.inputTarget.focus();
  }

  async executeCommand() {
    const command = this.inputTarget.value.trim();

    if (!command) {
      return;
    }

    // Add command to history display
    this.addToHistory(`sqlite> ${command}`, "user-command");

    // Show loading indicator
    const loadingElement = this.addToHistory("Executing...", "loading");

    // Clear the input
    this.inputTarget.value = "";

    // Store command in history
    this.commandHistory.push(command);

    try {
      const response = await fetch("/admin/sqlite/execute", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ query: command }),
      });

      // Remove loading indicator
      loadingElement.remove();

      if (response.ok) {
        const result: SqliteResult = await response.json();

        if (result.success) {
          if (result.output.trim()) {
            this.addToHistory(result.output, "command-output");
          } else {
            this.addToHistory("Query executed successfully.", "command-output");
          }
        } else {
          const errorMsg = result.error || "Unknown error occurred";
          this.addToHistory(`Error: ${errorMsg}`, "command-error");
        }
      } else {
        loadingElement.remove();
        this.addToHistory("Error: Failed to execute command", "command-error");
      }
    } catch (error) {
      loadingElement.remove();
      this.addToHistory(`Error: ${(error as Error).message}`, "command-error");
    }

    // Scroll to bottom
    this.scrollToBottom();
  }

  clear() {
    // Clear the history display but keep the welcome message
    this.historyTarget.innerHTML = `
            <div class="welcome-message">
                <p>SQLite Command Line Interface</p>
                <p>Type your SQL commands below. Examples:</p>
                <p class="example-cmd">.tables</p>
                <p class="example-cmd">.schema table_name</p>
                <p class="example-cmd">SELECT * FROM users LIMIT 10;</p>
            </div>
        `;

    // Clear command history
    this.commandHistory = [];

    // Focus input
    this.inputTarget.focus();
  }

  private addToHistory(content: string, className: string): HTMLElement {
    const element = document.createElement("div");
    element.className = className;
    element.textContent = content;

    this.historyTarget.appendChild(element);
    this.scrollToBottom();

    return element;
  }

  private scrollToBottom() {
    this.terminalTarget.scrollTop = this.terminalTarget.scrollHeight;
  }
}
