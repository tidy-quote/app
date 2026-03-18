import { Component, type ErrorInfo, type ReactNode } from "react";

interface ErrorBoundaryProps {
  children: ReactNode;
  fallback?: "page" | "app";
}

interface ErrorBoundaryState {
  hasError: boolean;
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(): ErrorBoundaryState {
    return { hasError: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    console.error("ErrorBoundary caught:", error, info.componentStack);
  }

  render(): ReactNode {
    if (!this.state.hasError) {
      return this.props.children;
    }

    const isAppLevel = this.props.fallback === "app";

    return (
      <div className="error-fallback">
        <h2 className="error-fallback-title">Something went wrong</h2>
        <p className="error-fallback-message">
          {isAppLevel
            ? "The app encountered an unexpected error."
            : "This page encountered an unexpected error."}
        </p>
        <button
          type="button"
          className="error-fallback-btn"
          onClick={() => {
            if (isAppLevel) {
              window.location.reload();
            } else {
              this.setState({ hasError: false });
            }
          }}
        >
          {isAppLevel ? "Reload app" : "Try again"}
        </button>
      </div>
    );
  }
}
