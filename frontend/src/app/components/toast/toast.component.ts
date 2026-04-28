import { Component, Input } from "@angular/core";
import { CommonModule } from "@angular/common";
import { ToastType } from "../../services/toast.service";

@Component({
  selector: "app-toast",
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="toast" [class]="type">
      <div class="toast-icon">
        <ng-container [ngSwitch]="type">
          <svg *ngSwitchCase="'success'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
            <polyline points="22 4 12 14.01 9 11.01" />
          </svg>
          <svg *ngSwitchCase="'error'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <line x1="15" y1="9" x2="9" y2="15" />
            <line x1="9" y1="9" x2="15" y2="15" />
          </svg>
          <svg *ngSwitchCase="'warning'" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
            <line x1="12" y1="9" x2="12" y2="13" />
            <line x1="12" y1="17" x2="12.01" y2="17" />
          </svg>
          <svg *ngSwitchDefault width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="16" x2="12" y2="12" />
            <line x1="12" y1="8" x2="12.01" y2="8" />
          </svg>
        </ng-container>
      </div>
      <div class="toast-message">{{ message }}</div>
      <button class="toast-close" (click)="onClose()">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>
  `,
  styles: [
    `
      .toast {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.75rem 1rem;
        border-radius: var(--radius-md);
        box-shadow: var(--shadow-lg);
        min-width: 280px;
        max-width: 400px;
        animation: slideIn 0.3s ease;
        border: 1px solid;

        &.success {
          background: #f0fdf4;
          border-color: #86efac;
          color: #166534;

          .toast-icon {
            color: #22c55e;
          }
        }

        &.error {
          background: #fef2f2;
          border-color: #fecaca;
          color: #991b1b;

          .toast-icon {
            color: #ef4444;
          }
        }

        &.warning {
          background: #fffbeb;
          border-color: #fed7aa;
          color: #92400e;

          .toast-icon {
            color: #f59e0b;
          }
        }

        &.info {
          background: #eff6ff;
          border-color: #bfdbfe;
          color: #1e40af;

          .toast-icon {
            color: #3b82f6;
          }
        }
      }

      .toast-message {
        flex: 1;
        font-size: 0.875rem;
        line-height: 1.4;
      }

      .toast-close {
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 0.25rem;
        border: none;
        background: transparent;
        cursor: pointer;
        color: inherit;
        opacity: 0.6;
        border-radius: var(--radius-sm);
        transition: all var(--transition-fast);

        &:hover {
          opacity: 1;
          background: rgba(0, 0, 0, 0.05);
        }
      }

      @keyframes slideIn {
        from {
          opacity: 0;
          transform: translateX(100%);
        }
        to {
          opacity: 1;
          transform: translateX(0);
        }
      }
    `,
  ],
})
export class ToastComponent {
  @Input() id!: number;
  @Input() type: ToastType = "info";
  @Input() message: string = "";

  onClose: () => void = () => {};
}
