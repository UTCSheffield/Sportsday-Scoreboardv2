import { Controller } from "@hotwired/stimulus";

export default class extends Controller<HTMLElement> {
  static override targets = ["output", "autoScroll"];

  declare readonly outputTarget: HTMLElement;
  declare readonly autoScrollTarget: HTMLInputElement;
  declare readonly hasAutoScrollTarget: boolean;

  private refreshInterval?: number;

  override connect() {
    // Auto-scroll to bottom if checkbox is checked
    this.autoScrollToBottom();

    // Start auto-refresh
    this.startAutoRefresh();

    // Handle visibility change
    document.addEventListener(
      "visibilitychange",
      this.handleVisibilityChange.bind(this),
    );
  }

  override disconnect() {
    this.stopAutoRefresh();
    document.removeEventListener(
      "visibilitychange",
      this.handleVisibilityChange.bind(this),
    );
  }

  refresh() {
    window.location.reload();
  }

  async clear() {
    if (!confirm("Are you sure you want to clear all logs?")) {
      return;
    }

    try {
      const response = await fetch("/admin/console/clear", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
      });

      if (response.ok) {
        window.location.reload();
      } else {
        alert("Failed to clear logs");
      }
    } catch (error) {
      alert("Error clearing logs: " + (error as Error).message);
    }
  }

  toggleAutoScroll() {
    if (this.autoScrollTarget.checked) {
      this.autoScrollToBottom();
    }
  }

  private autoScrollToBottom() {
    if (this.hasAutoScrollTarget && this.autoScrollTarget.checked) {
      this.outputTarget.scrollTop = this.outputTarget.scrollHeight;
    }
  }

  private startAutoRefresh() {
    this.refreshInterval = window.setInterval(() => {
      if (!document.hidden) {
        this.refresh();
      }
    }, 30000); // 30 seconds
  }

  private stopAutoRefresh() {
    if (this.refreshInterval) {
      clearInterval(this.refreshInterval);
      this.refreshInterval = undefined;
    }
  }

  private handleVisibilityChange() {
    if (document.hidden) {
      this.stopAutoRefresh();
    } else {
      this.startAutoRefresh();
    }
  }
}
