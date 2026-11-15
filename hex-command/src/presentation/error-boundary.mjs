import { Component } from "react";

// React error boundaries require a class; keep the runtime bridge here while
// Reason owns the boundary's typed props and the startup failure presentation.
export class ErrorBoundary extends Component {
  state = { failed: false };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  componentDidCatch(error) {
    this.props.onError(error);
  }

  render() {
    return this.state.failed ? null : this.props.render();
  }
}
