import { Component } from "@angular/core";
import { RouterOutlet } from "@angular/router";
import { CommonModule } from "@angular/common";
import { Router } from "@angular/router";
import { AuthService } from "./services/auth.service";
import { Observable } from "rxjs";
import { User } from "./models";

@Component({
  selector: "app-root",
  standalone: true,
  imports: [RouterOutlet, CommonModule],
  template: `
    <nav class="navbar" *ngIf="isAuthenticated$ | async">
      <div class="navbar-brand">
        <a routerLink="/boards" class="logo">
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <rect x="3" y="3" width="7" height="18" rx="1" />
            <rect x="14" y="3" width="7" height="12" rx="1" />
          </svg>
          <span>Kanban</span>
        </a>
      </div>
      <div class="navbar-user">
        <span class="username">{{ (currentUser$ | async)?.username }}</span>
        <button class="btn btn-sm btn-secondary" (click)="logout()">
          退出
        </button>
      </div>
    </nav>
    <main class="main-content">
      <router-outlet />
    </main>
  `,
  styles: [
    `
      .navbar {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0.75rem 1.5rem;
        background: var(--bg-primary);
        border-bottom: 1px solid var(--border-color);
        box-shadow: var(--shadow-sm);
      }

      .navbar-brand {
        display: flex;
        align-items: center;
      }

      .logo {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        color: var(--text-primary);
        text-decoration: none;
        font-size: 1.25rem;
        font-weight: 600;

        &:hover {
          text-decoration: none;
        }
      }

      .navbar-user {
        display: flex;
        align-items: center;
        gap: 1rem;
      }

      .username {
        color: var(--text-secondary);
        font-size: 0.875rem;
      }

      .main-content {
        min-height: calc(100vh - 57px);
      }

      .btn {
        padding: 0.5rem 1rem;
        border: none;
        border-radius: var(--radius-md);
        font-size: 0.875rem;
        font-weight: 500;
        cursor: pointer;
        transition: all var(--transition-fast);
      }

      .btn-sm {
        padding: 0.375rem 0.75rem;
        font-size: 0.75rem;
      }

      .btn-secondary {
        background: var(--bg-tertiary);
        color: var(--text-primary);

        &:hover {
          background: var(--bg-secondary);
        }
      }
    `,
  ],
})
export class AppComponent {
  isAuthenticated$: Observable<boolean>;
  currentUser$: Observable<User | null>;

  constructor(
    private authService: AuthService,
    private router: Router,
  ) {
    this.isAuthenticated$ = this.authService.currentUser$.pipe((user) =>
      user.pipe(map((u) => !!u)),
    );
    this.currentUser$ = this.authService.currentUser$;
  }

  logout(): void {
    this.authService.logout();
    this.router.navigate(["/login"]);
  }
}

import { map } from "rxjs/operators";
