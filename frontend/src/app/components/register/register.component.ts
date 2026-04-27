import { Component } from "@angular/core";
import { CommonModule } from "@angular/common";
import {
  FormBuilder,
  FormGroup,
  ReactiveFormsModule,
  Validators,
  AbstractControl,
  ValidationErrors,
  ValidatorFn,
} from "@angular/forms";
import { Router, RouterLink } from "@angular/router";
import { AuthService } from "../../services/auth.service";
import { finalize } from "rxjs/operators";

export const passwordMatchValidator: ValidatorFn = (
  control: AbstractControl,
): ValidationErrors | null => {
  const password = control.get("password");
  const confirmPassword = control.get("confirmPassword");

  if (password && confirmPassword && password.value !== confirmPassword.value) {
    return { passwordMismatch: true };
  }
  return null;
};

@Component({
  selector: "app-register",
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule, RouterLink],
  template: `
    <div class="auth-container">
      <div class="auth-card">
        <div class="auth-header">
          <svg
            width="48"
            height="48"
            viewBox="0 0 24 24"
            fill="none"
            stroke="#3b82f6"
            stroke-width="2"
          >
            <rect x="3" y="3" width="7" height="18" rx="1" />
            <rect x="14" y="3" width="7" height="12" rx="1" />
          </svg>
          <h1>创建账户</h1>
          <p>开始使用多人协作看板</p>
        </div>

        <form
          [formGroup]="registerForm"
          (ngSubmit)="onSubmit()"
          class="auth-form"
        >
          <div class="form-group">
            <label for="username">用户名</label>
            <input
              type="text"
              id="username"
              formControlName="username"
              placeholder="请输入用户名"
              [class.error]="
                username.invalid && (username.dirty || username.touched)
              "
            />
            <div
              *ngIf="username.invalid && (username.dirty || username.touched)"
              class="error-message"
            >
              <span *ngIf="username.errors?.['required']">请输入用户名</span>
              <span *ngIf="username.errors?.['minlength']"
                >用户名至少3个字符</span
              >
              <span *ngIf="username.errors?.['maxlength']"
                >用户名最多50个字符</span
              >
            </div>
          </div>

          <div class="form-group">
            <label for="email">邮箱</label>
            <input
              type="email"
              id="email"
              formControlName="email"
              placeholder="请输入邮箱"
              [class.error]="email.invalid && (email.dirty || email.touched)"
            />
            <div
              *ngIf="email.invalid && (email.dirty || email.touched)"
              class="error-message"
            >
              <span *ngIf="email.errors?.['required']">请输入邮箱</span>
              <span *ngIf="email.errors?.['email']">请输入有效的邮箱地址</span>
            </div>
          </div>

          <div class="form-group">
            <label for="password">密码</label>
            <input
              type="password"
              id="password"
              formControlName="password"
              placeholder="请输入密码"
              [class.error]="
                password.invalid && (password.dirty || password.touched)
              "
            />
            <div
              *ngIf="password.invalid && (password.dirty || password.touched)"
              class="error-message"
            >
              <span *ngIf="password.errors?.['required']">请输入密码</span>
              <span *ngIf="password.errors?.['minlength']"
                >密码至少6个字符</span
              >
            </div>
          </div>

          <div class="form-group">
            <label for="confirmPassword">确认密码</label>
            <input
              type="password"
              id="confirmPassword"
              formControlName="confirmPassword"
              placeholder="请再次输入密码"
              [class.error]="
                confirmPassword.invalid &&
                (confirmPassword.dirty || confirmPassword.touched)
              "
            />
            <div
              *ngIf="
                (confirmPassword.invalid ||
                  registerForm.hasError('passwordMismatch')) &&
                (confirmPassword.dirty || confirmPassword.touched)
              "
              class="error-message"
            >
              <span *ngIf="confirmPassword.errors?.['required']"
                >请确认密码</span
              >
              <span *ngIf="registerForm.hasError('passwordMismatch')"
                >两次密码输入不一致</span
              >
            </div>
          </div>

          <div *ngIf="errorMessage" class="error-alert">
            {{ errorMessage }}
          </div>

          <button
            type="submit"
            class="btn btn-primary btn-block"
            [disabled]="loading || registerForm.invalid"
          >
            {{ loading ? "注册中..." : "注册" }}
          </button>
        </form>

        <div class="auth-footer">
          已有账户？<a routerLink="/login">立即登录</a>
        </div>
      </div>
    </div>
  `,
  styles: [
    `
      .auth-container {
        min-height: 100vh;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 1rem;
        background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      }

      .auth-card {
        width: 100%;
        max-width: 420px;
        background: var(--bg-primary);
        border-radius: var(--radius-lg);
        padding: 2rem;
        box-shadow: var(--shadow-lg);
      }

      .auth-header {
        text-align: center;
        margin-bottom: 2rem;

        h1 {
          margin: 1rem 0 0.5rem 0;
          font-size: 1.5rem;
        }

        p {
          margin: 0;
          color: var(--text-secondary);
        }
      }

      .auth-form {
        .form-group {
          margin-bottom: 1.25rem;

          label {
            display: block;
            margin-bottom: 0.5rem;
            font-weight: 500;
            font-size: 0.875rem;
          }

          input {
            width: 100%;
            padding: 0.75rem 1rem;
            border: 1px solid var(--border-color);
            border-radius: var(--radius-md);
            font-size: 1rem;
            transition:
              border-color var(--transition-fast),
              box-shadow var(--transition-fast);

            &:focus {
              outline: none;
              border-color: var(--primary-color);
              box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
            }

            &.error {
              border-color: var(--danger-color);
            }
          }

          .error-message {
            margin-top: 0.25rem;
            font-size: 0.75rem;
            color: var(--danger-color);
          }
        }
      }

      .error-alert {
        padding: 0.75rem 1rem;
        background: #fef2f2;
        border: 1px solid #fecaca;
        border-radius: var(--radius-md);
        color: var(--danger-color);
        font-size: 0.875rem;
        margin-bottom: 1rem;
      }

      .btn {
        padding: 0.75rem 1.5rem;
        border: none;
        border-radius: var(--radius-md);
        font-size: 1rem;
        font-weight: 500;
        cursor: pointer;
        transition: all var(--transition-fast);

        &:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }
      }

      .btn-primary {
        background: var(--primary-color);
        color: white;

        &:hover:not(:disabled) {
          background: var(--primary-hover);
        }
      }

      .btn-block {
        width: 100%;
      }

      .auth-footer {
        margin-top: 1.5rem;
        text-align: center;
        font-size: 0.875rem;
        color: var(--text-secondary);

        a {
          color: var(--primary-color);
          text-decoration: none;
          font-weight: 500;

          &:hover {
            text-decoration: underline;
          }
        }
      }
    `,
  ],
})
export class RegisterComponent {
  registerForm: FormGroup;
  loading = false;
  errorMessage = "";

  constructor(
    private fb: FormBuilder,
    private authService: AuthService,
    private router: Router,
  ) {
    this.registerForm = this.fb.group(
      {
        username: [
          "",
          [
            Validators.required,
            Validators.minLength(3),
            Validators.maxLength(50),
          ],
        ],
        email: ["", [Validators.required, Validators.email]],
        password: ["", [Validators.required, Validators.minLength(6)]],
        confirmPassword: ["", [Validators.required]],
      },
      { validators: passwordMatchValidator },
    );
  }

  get username() {
    return this.registerForm.get("username")!;
  }

  get email() {
    return this.registerForm.get("email")!;
  }

  get password() {
    return this.registerForm.get("password")!;
  }

  get confirmPassword() {
    return this.registerForm.get("confirmPassword")!;
  }

  onSubmit(): void {
    if (this.registerForm.invalid) {
      return;
    }

    this.loading = true;
    this.errorMessage = "";

    const { confirmPassword, ...registerData } = this.registerForm.value;

    this.authService
      .register(registerData)
      .pipe(finalize(() => (this.loading = false)))
      .subscribe({
        next: () => {
          this.router.navigate(["/boards"]);
        },
        error: (err) => {
          if (err.status === 409) {
            this.errorMessage = "用户名或邮箱已被注册";
          } else {
            this.errorMessage = "注册失败，请稍后重试";
          }
        },
      });
  }
}
