import { Injectable, NgZone, ApplicationRef, ComponentRef, createComponent, EnvironmentInjector } from "@angular/core";
import { ToastComponent } from "../components/toast/toast.component";

export type ToastType = "success" | "error" | "info" | "warning";

export interface ToastOptions {
  type: ToastType;
  message: string;
  duration?: number;
}

@Injectable({
  providedIn: "root",
})
export class ToastService {
  private container: HTMLElement | null = null;
  private toastComponents: Map<number, ComponentRef<ToastComponent>> = new Map();
  private toastIdCounter = 0;

  constructor(
    private zone: NgZone,
    private appRef: ApplicationRef,
    private injector: EnvironmentInjector,
  ) {}

  private getContainer(): HTMLElement {
    if (!this.container) {
      this.container = document.createElement("div");
      this.container.className = "toast-container";
      this.container.style.position = "fixed";
      this.container.style.top = "1rem";
      this.container.style.right = "1rem";
      this.container.style.zIndex = "9999";
      this.container.style.display = "flex";
      this.container.style.flexDirection = "column";
      this.container.style.gap = "0.5rem";
      document.body.appendChild(this.container);
    }
    return this.container;
  }

  show(options: ToastOptions): void {
    this.zone.run(() => {
      const id = ++this.toastIdCounter;
      const duration = options.duration || 3000;

      const componentRef = createComponent(ToastComponent, {
        environmentInjector: this.injector,
      });

      componentRef.instance.type = options.type;
      componentRef.instance.message = options.message;
      componentRef.instance.id = id;

      this.appRef.attachView(componentRef.hostView);
      const domElem = (componentRef.hostView as any).rootNodes[0] as HTMLElement;
      this.getContainer().appendChild(domElem);

      this.toastComponents.set(id, componentRef);

      const timeout = setTimeout(() => {
        this.removeToast(id);
      }, duration);

      componentRef.instance.onClose = () => {
        clearTimeout(timeout);
        this.removeToast(id);
      };
    });
  }

  private removeToast(id: number): void {
    this.zone.run(() => {
      const componentRef = this.toastComponents.get(id);
      if (componentRef) {
        const domElem = (componentRef.hostView as any).rootNodes[0] as HTMLElement;
        domElem.style.opacity = "0";
        domElem.style.transform = "translateX(100%)";
        domElem.style.transition = "all 0.3s ease";

        setTimeout(() => {
          if (this.container) {
            try {
              this.container.removeChild(domElem);
            } catch (e) {
              // ignore
            }
          }
          this.appRef.detachView(componentRef.hostView);
          componentRef.destroy();
          this.toastComponents.delete(id);
        }, 300);
      }
    });
  }

  success(message: string, duration?: number): void {
    this.show({ type: "success", message, duration });
  }

  error(message: string, duration?: number): void {
    this.show({ type: "error", message, duration });
  }

  info(message: string, duration?: number): void {
    this.show({ type: "info", message, duration });
  }

  warning(message: string, duration?: number): void {
    this.show({ type: "warning", message, duration });
  }
}
