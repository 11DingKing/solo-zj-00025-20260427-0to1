import { Component, OnInit } from "@angular/core";
import { CommonModule } from "@angular/common";
import {
  FormBuilder,
  FormGroup,
  ReactiveFormsModule,
  Validators,
} from "@angular/forms";
import { Router } from "@angular/router";
import { ApiService } from "../../services/api.service";
import { Board } from "../../models";
import {
  finalize,
  debounceTime,
  distinctUntilChanged,
  switchMap,
} from "rxjs/operators";
import { Subject, Observable } from "rxjs";

@Component({
  selector: "app-board-list",
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule],
  template: `
    <div class="board-list-container">
      <div class="page-header">
        <div class="header-left">
          <h1>我的看板</h1>
          <div class="search-box">
            <svg
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
            <input
              type="text"
              placeholder="搜索看板..."
              (input)="onSearch($event)"
            />
          </div>
        </div>
        <button class="btn btn-primary" (click)="showCreateModal = true">
          <svg
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <line x1="12" y1="5" x2="12" y2="19" />
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
          新建看板
        </button>
      </div>

      <div *ngIf="loading" class="loading-state">
        <div class="spinner"></div>
        <p>加载中...</p>
      </div>

      <ng-container *ngIf="!loading">
        <div *ngIf="boards.length === 0" class="empty-state">
          <svg
            width="64"
            height="64"
            viewBox="0 0 24 24"
            fill="none"
            stroke="var(--secondary-color)"
            stroke-width="1.5"
          >
            <rect x="3" y="3" width="7" height="18" rx="1" />
            <rect x="14" y="3" width="7" height="12" rx="1" />
          </svg>
          <h3>还没有看板</h3>
          <p>创建您的第一个看板，开始协作吧</p>
          <button class="btn btn-primary" (click)="showCreateModal = true">
            创建看板
          </button>
        </div>

        <div *ngIf="boards.length > 0" class="boards-grid">
          <div
            *ngFor="let board of boards"
            class="board-card"
            (click)="goToBoard(board)"
          >
            <div class="board-card-header">
              <h3>{{ board.name }}</h3>
              <div class="board-actions" (click)="$event.stopPropagation()">
                <button
                  class="btn-icon"
                  (click)="openEditBoard(board)"
                  title="编辑"
                >
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                  >
                    <path
                      d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"
                    />
                    <path
                      d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"
                    />
                  </svg>
                </button>
              </div>
            </div>
            <p *ngIf="board.description" class="board-description">
              {{ board.description }}
            </p>
            <div class="board-footer">
              <span class="board-date">{{ formatDate(board.created_at) }}</span>
            </div>
          </div>
        </div>
      </ng-container>
    </div>

    <div
      *ngIf="showCreateModal || showEditModal"
      class="modal-overlay"
      (click)="closeModal()"
    >
      <div class="modal" (click)="$event.stopPropagation()">
        <div class="modal-header">
          <h2>{{ showEditModal ? "编辑看板" : "创建看板" }}</h2>
          <button class="btn-icon" (click)="closeModal()">
            <svg
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        <form [formGroup]="boardForm" (ngSubmit)="onSubmit()">
          <div class="form-group">
            <label for="name">看板名称</label>
            <input
              type="text"
              id="name"
              formControlName="name"
              placeholder="输入看板名称"
            />
            <div
              *ngIf="name.invalid && (name.dirty || name.touched)"
              class="error-message"
            >
              请输入看板名称
            </div>
          </div>
          <div class="form-group">
            <label for="description">描述 (可选)</label>
            <textarea
              id="description"
              formControlName="description"
              placeholder="输入看板描述"
              rows="3"
            ></textarea>
          </div>
          <div class="modal-actions">
            <button
              type="button"
              class="btn btn-secondary"
              (click)="closeModal()"
            >
              取消
            </button>
            <button
              type="submit"
              class="btn btn-primary"
              [disabled]="submitting || boardForm.invalid"
            >
              {{ submitting ? "处理中..." : showEditModal ? "保存" : "创建" }}
            </button>
          </div>
        </form>
      </div>
    </div>
  `,
  styles: [
    `
      .board-list-container {
        max-width: 1200px;
        margin: 0 auto;
        padding: 2rem;
      }

      .page-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 2rem;
      }

      .header-left {
        display: flex;
        align-items: center;
        gap: 1.5rem;
      }

      .search-box {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 1rem;
        background: var(--bg-primary);
        border: 1px solid var(--border-color);
        border-radius: var(--radius-md);
        color: var(--text-secondary);
        width: 300px;

        input {
          border: none;
          outline: none;
          background: transparent;
          width: 100%;
          font-size: 0.875rem;
          color: var(--text-primary);

          &::placeholder {
            color: var(--text-secondary);
          }
        }
      }

      .boards-grid {
        display: grid;
        grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
        gap: 1.5rem;
      }

      .board-card {
        background: var(--bg-primary);
        border-radius: var(--radius-md);
        padding: 1.5rem;
        cursor: pointer;
        transition: all var(--transition-normal);
        border: 1px solid var(--border-color);

        &:hover {
          box-shadow: var(--shadow-md);
          transform: translateY(-2px);
        }
      }

      .board-card-header {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
        gap: 1rem;
        margin-bottom: 0.75rem;

        h3 {
          font-size: 1.125rem;
          margin: 0;
          word-break: break-word;
        }
      }

      .board-description {
        color: var(--text-secondary);
        font-size: 0.875rem;
        margin-bottom: 1rem;
        word-break: break-word;
        display: -webkit-box;
        -webkit-line-clamp: 2;
        -webkit-box-orient: vertical;
        overflow: hidden;
      }

      .board-footer {
        display: flex;
        align-items: center;
        justify-content: space-between;
      }

      .board-date {
        color: var(--text-secondary);
        font-size: 0.75rem;
      }

      .loading-state,
      .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 4rem;
        color: var(--text-secondary);
      }

      .spinner {
        width: 40px;
        height: 40px;
        border: 3px solid var(--bg-tertiary);
        border-top-color: var(--primary-color);
        border-radius: 50%;
        animation: spin 1s linear infinite;
      }

      @keyframes spin {
        to {
          transform: rotate(360deg);
        }
      }

      .empty-state {
        h3 {
          margin: 1rem 0 0.5rem 0;
          color: var(--text-primary);
        }

        p {
          margin-bottom: 1.5rem;
        }
      }

      .btn {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.625rem 1.25rem;
        border: none;
        border-radius: var(--radius-md);
        font-size: 0.875rem;
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

      .btn-secondary {
        background: var(--bg-tertiary);
        color: var(--text-primary);

        &:hover:not(:disabled) {
          background: var(--bg-secondary);
        }
      }

      .btn-icon {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 0.375rem;
        border: none;
        background: transparent;
        border-radius: var(--radius-sm);
        cursor: pointer;
        color: var(--text-secondary);
        transition: all var(--transition-fast);

        &:hover {
          background: var(--bg-secondary);
          color: var(--text-primary);
        }
      }

      .modal-overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
        padding: 1rem;
      }

      .modal {
        background: var(--bg-primary);
        border-radius: var(--radius-lg);
        width: 100%;
        max-width: 500px;
        max-height: 90vh;
        overflow: auto;
      }

      .modal-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 1.5rem;
        border-bottom: 1px solid var(--border-color);

        h2 {
          margin: 0;
          font-size: 1.25rem;
        }
      }

      form {
        padding: 1.5rem;
      }

      .form-group {
        margin-bottom: 1.25rem;

        label {
          display: block;
          margin-bottom: 0.5rem;
          font-weight: 500;
          font-size: 0.875rem;
        }

        input,
        textarea {
          width: 100%;
          padding: 0.75rem 1rem;
          border: 1px solid var(--border-color);
          border-radius: var(--radius-md);
          font-size: 0.875rem;
          font-family: inherit;
          transition:
            border-color var(--transition-fast),
            box-shadow var(--transition-fast);

          &:focus {
            outline: none;
            border-color: var(--primary-color);
            box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
          }
        }

        textarea {
          resize: vertical;
          min-height: 80px;
        }

        .error-message {
          margin-top: 0.25rem;
          font-size: 0.75rem;
          color: var(--danger-color);
        }
      }

      .modal-actions {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 0.75rem;
        margin-top: 1.5rem;
      }
    `,
  ],
})
export class BoardListComponent implements OnInit {
  boards: Board[] = [];
  loading = false;
  showCreateModal = false;
  showEditModal = false;
  editingBoard: Board | null = null;
  boardForm: FormGroup;
  submitting = false;

  private searchSubject = new Subject<string>();

  constructor(
    private apiService: ApiService,
    private router: Router,
    private fb: FormBuilder,
  ) {
    this.boardForm = this.fb.group({
      name: ["", [Validators.required, Validators.maxLength(100)]],
      description: [""],
    });
  }

  ngOnInit(): void {
    this.loadBoards();

    this.searchSubject
      .pipe(
        debounceTime(300),
        distinctUntilChanged(),
        switchMap((search) => this.apiService.getBoards(search)),
      )
      .subscribe({
        next: (boards) => {
          this.boards = boards;
        },
      });
  }

  get name() {
    return this.boardForm.get("name")!;
  }

  loadBoards(): void {
    this.loading = true;
    this.apiService
      .getBoards()
      .pipe(finalize(() => (this.loading = false)))
      .subscribe({
        next: (boards) => {
          this.boards = boards;
        },
      });
  }

  onSearch(event: Event): void {
    const input = event.target as HTMLInputElement;
    this.searchSubject.next(input.value);
  }

  goToBoard(board: Board): void {
    this.router.navigate(["/boards", board.id]);
  }

  openEditBoard(board: Board): void {
    this.editingBoard = board;
    this.boardForm.patchValue({
      name: board.name,
      description: board.description || "",
    });
    this.showEditModal = true;
  }

  closeModal(): void {
    this.showCreateModal = false;
    this.showEditModal = false;
    this.editingBoard = null;
    this.boardForm.reset();
  }

  onSubmit(): void {
    if (this.boardForm.invalid) {
      return;
    }

    this.submitting = true;

    const request =
      this.showEditModal && this.editingBoard
        ? this.apiService.updateBoard(
            this.editingBoard.id,
            this.boardForm.value,
          )
        : this.apiService.createBoard(this.boardForm.value);

    request.pipe(finalize(() => (this.submitting = false))).subscribe({
      next: () => {
        this.loadBoards();
        this.closeModal();
      },
    });
  }

  formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString("zh-CN", {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  }
}
