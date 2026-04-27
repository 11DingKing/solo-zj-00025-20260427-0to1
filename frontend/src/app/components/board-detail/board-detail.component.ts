import { Component, OnInit, OnDestroy } from "@angular/core";
import { CommonModule } from "@angular/common";
import {
  FormBuilder,
  FormGroup,
  ReactiveFormsModule,
  Validators,
} from "@angular/forms";
import { ActivatedRoute, Router } from "@angular/router";
import {
  CdkDragDrop,
  CdkDropList,
  CdkDrag,
  moveItemInArray,
  transferArrayItem,
} from "@angular/cdk/drag-drop";
import { ApiService } from "../../services/api.service";
import {
  Board,
  Column,
  Card,
  Tag,
  BoardMember,
  ActivityWithUser,
  Priority,
} from "../../models";
import { finalize, forkJoin, Subject, takeUntil } from "rxjs";

interface ColumnWithCards extends Column {
  cards: Card[];
}

@Component({
  selector: "app-board-detail",
  standalone: true,
  imports: [CommonModule, ReactiveFormsModule, CdkDropList, CdkDrag],
  template: `
    <div class="board-detail-container">
      <div class="board-header" *ngIf="board">
        <div class="header-left">
          <button class="btn btn-sm btn-secondary" routerLink="/boards">
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <polyline points="15 18 9 12 15 6" />
            </svg>
            返回
          </button>
          <h1>{{ board.name }}</h1>
        </div>
        <div class="header-actions">
          <button
            class="btn btn-sm"
            [class.active]="showActivityPanel"
            (click)="toggleActivityPanel()"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <circle cx="12" cy="12" r="10" />
              <polyline points="12 6 12 12 16 14" />
            </svg>
            活动
          </button>
          <button
            class="btn btn-sm"
            [class.active]="showMembersPanel"
            (click)="toggleMembersPanel()"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2" />
              <circle cx="9" cy="7" r="4" />
              <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
              <path d="M16 3.13a4 4 0 0 1 0 7.75" />
            </svg>
            成员
          </button>
          <button
            class="btn btn-sm btn-primary"
            (click)="showCreateColumnModal = true"
          >
            <svg
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <line x1="12" y1="5" x2="12" y2="19" />
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
            添加列
          </button>
        </div>
      </div>

      <div class="board-content">
        <div class="columns-container" cdkDropListOrientation="horizontal">
          <ng-container *ngIf="columns.length > 0">
            <div
              *ngFor="let column of columns; trackBy: trackByColumn"
              class="column-wrapper"
              cdkDrag
              [cdkDragData]="column"
              (cdkDragDropped)="dropColumn($event)"
            >
              <div class="column" *ngIf="column">
                <div class="column-header" cdkDragHandle>
                  <h3>{{ column.name }}</h3>
                  <div class="column-actions">
                    <button
                      class="btn-icon"
                      (click)="addCard(column)"
                      title="添加卡片"
                    >
                      <svg
                        width="16"
                        height="16"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                      >
                        <line x1="12" y1="5" x2="12" y2="19" />
                        <line x1="5" y1="12" x2="19" y2="12" />
                      </svg>
                    </button>
                    <button
                      class="btn-icon"
                      (click)="openEditColumn(column)"
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

                <div
                  class="cards-container"
                  cdkDropList
                  [cdkDropListData]="column.cards"
                  (cdkDropListDropped)="dropCard($event)"
                  [cdkDropListConnectedTo]="connectedLists"
                >
                  <div
                    *ngFor="let card of column.cards; trackBy: trackByCard"
                    class="card"
                    cdkDrag
                    [cdkDragData]="{ card, columnId: column.id }"
                    (click)="openCardDetail(card, column)"
                  >
                    <div class="card-header">
                      <h4>{{ card.title }}</h4>
                      <span
                        class="priority-badge"
                        [class]="getPriorityClass(card.priority)"
                      >
                        {{ card.priority }}
                      </span>
                    </div>
                    <p *ngIf="card.description" class="card-description">
                      {{ card.description | slice: 0 : 100
                      }}{{ card.description?.length! > 100 ? "..." : "" }}
                    </p>
                    <div
                      class="card-footer"
                      *ngIf="card.due_date || card.assignee_id"
                    >
                      <div class="card-due" *ngIf="card.due_date">
                        <svg
                          width="14"
                          height="14"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                        >
                          <rect
                            x="3"
                            y="4"
                            width="18"
                            height="18"
                            rx="2"
                            ry="2"
                          />
                          <line x1="16" y1="2" x2="16" y2="6" />
                          <line x1="8" y1="2" x2="8" y2="6" />
                          <line x1="3" y1="10" x2="21" y2="10" />
                        </svg>
                        <span>{{ formatDate(card.due_date) }}</span>
                      </div>
                      <div class="card-assignee" *ngIf="card.assignee_id">
                        <div class="avatar">
                          {{ getAvatarInitials(card.assignee_id) }}
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <button class="add-card-btn" (click)="addCard(column)">
                  <svg
                    width="16"
                    height="16"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                  >
                    <line x1="12" y1="5" x2="12" y2="19" />
                    <line x1="5" y1="12" x2="19" y2="12" />
                  </svg>
                  添加卡片
                </button>
              </div>
            </div>
          </ng-container>

          <div class="empty-columns" *ngIf="columns.length === 0 && !loading">
            <p>还没有列，点击上方按钮添加第一列</p>
          </div>
        </div>

        <div
          class="side-panel"
          [class.active]="showActivityPanel || showMembersPanel"
        >
          <div class="activity-panel" *ngIf="showActivityPanel">
            <div class="panel-header">
              <h3>活动日志</h3>
              <button class="btn-icon" (click)="toggleActivityPanel()">
                <svg
                  width="20"
                  height="20"
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
            <div class="activity-list">
              <div *ngIf="activities.length === 0" class="empty-activities">
                暂无活动记录
              </div>
              <div *ngFor="let activity of activities" class="activity-item">
                <div class="activity-avatar">
                  {{ activity.user.username.charAt(0).toUpperCase() }}
                </div>
                <div class="activity-content">
                  <p class="activity-text">
                    <strong>{{ activity.user.username }}</strong>
                    {{ formatActivity(activity) }}
                  </p>
                  <p class="activity-time">
                    {{ formatRelativeTime(activity.activity.created_at) }}
                  </p>
                </div>
              </div>
            </div>
          </div>

          <div class="members-panel" *ngIf="showMembersPanel">
            <div class="panel-header">
              <h3>看板成员</h3>
              <button class="btn-icon" (click)="toggleMembersPanel()">
                <svg
                  width="20"
                  height="20"
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
            <div class="members-list">
              <div *ngFor="let member of members" class="member-item">
                <div class="member-avatar">
                  {{ member.username.charAt(0).toUpperCase() }}
                </div>
                <div class="member-info">
                  <p class="member-name">{{ member.username }}</p>
                  <p class="member-email">{{ member.email }}</p>
                </div>
                <span class="role-badge" [class]="member.role">
                  {{
                    member.role === "owner"
                      ? "所有者"
                      : member.role === "admin"
                        ? "管理员"
                        : "成员"
                  }}
                </span>
              </div>
            </div>
            <button
              class="btn btn-primary btn-block mt-4"
              (click)="showInviteModal = true"
            >
              邀请成员
            </button>
          </div>
        </div>
      </div>

      <div
        *ngIf="showCreateColumnModal || showEditColumnModal"
        class="modal-overlay"
        (click)="closeColumnModal()"
      >
        <div class="modal" (click)="$event.stopPropagation()">
          <div class="modal-header">
            <h2>{{ showEditColumnModal ? "编辑列" : "创建列" }}</h2>
            <button class="btn-icon" (click)="closeColumnModal()">
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
          <form [formGroup]="columnForm" (ngSubmit)="onSubmitColumn()">
            <div class="form-group">
              <label for="columnName">列名称</label>
              <input
                type="text"
                id="columnName"
                formControlName="name"
                placeholder="输入列名称"
              />
            </div>
            <div class="form-group">
              <label for="columnColor">颜色</label>
              <div class="color-picker">
                <div
                  *ngFor="let color of colorOptions"
                  class="color-option"
                  [class.selected]="columnForm.get('color')?.value === color"
                  [style.backgroundColor]="color"
                  (click)="columnForm.patchValue({ color })"
                ></div>
              </div>
            </div>
            <div class="modal-actions">
              <button
                type="button"
                class="btn btn-secondary"
                (click)="closeColumnModal()"
              >
                取消
              </button>
              <button
                type="submit"
                class="btn btn-primary"
                [disabled]="columnForm.invalid || submittingColumn"
              >
                {{
                  submittingColumn
                    ? "处理中..."
                    : showEditColumnModal
                      ? "保存"
                      : "创建"
                }}
              </button>
            </div>
          </form>
        </div>
      </div>

      <div
        *ngIf="showCreateCardModal"
        class="modal-overlay"
        (click)="closeCardModal()"
      >
        <div class="modal large-modal" (click)="$event.stopPropagation()">
          <div class="modal-header">
            <h2>{{ editingCard ? "编辑卡片" : "创建卡片" }}</h2>
            <button class="btn-icon" (click)="closeCardModal()">
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
          <form [formGroup]="cardForm" (ngSubmit)="onSubmitCard()">
            <div class="form-group">
              <label for="cardTitle">标题</label>
              <input
                type="text"
                id="cardTitle"
                formControlName="title"
                placeholder="输入卡片标题"
              />
            </div>
            <div class="form-group">
              <label for="cardDescription">描述 (支持 Markdown)</label>
              <textarea
                id="cardDescription"
                formControlName="description"
                placeholder="输入描述，支持 Markdown 格式"
                rows="5"
              ></textarea>
            </div>
            <div class="form-row">
              <div class="form-group">
                <label for="cardPriority">优先级</label>
                <select id="cardPriority" formControlName="priority">
                  <option value="P0">P0 - 紧急</option>
                  <option value="P1">P1 - 高</option>
                  <option value="P2">P2 - 中</option>
                  <option value="P3">P3 - 低</option>
                </select>
              </div>
              <div class="form-group">
                <label for="cardDueDate">截止日期</label>
                <input
                  type="datetime-local"
                  id="cardDueDate"
                  formControlName="due_date"
                />
              </div>
            </div>
            <div class="form-row">
              <div class="form-group">
                <label for="cardAssignee">指派人</label>
                <select id="cardAssignee" formControlName="assignee_id">
                  <option [ngValue]="null">未分配</option>
                  <option
                    *ngFor="let member of members"
                    [value]="member.user_id"
                  >
                    {{ member.username }}
                  </option>
                </select>
              </div>
            </div>
            <div class="modal-actions">
              <button
                type="button"
                class="btn btn-secondary"
                (click)="closeCardModal()"
              >
                取消
              </button>
              <button
                *ngIf="editingCard"
                type="button"
                class="btn btn-danger"
                (click)="deleteCard()"
              >
                删除
              </button>
              <button
                type="submit"
                class="btn btn-primary"
                [disabled]="cardForm.invalid || submittingCard"
              >
                {{
                  submittingCard ? "处理中..." : editingCard ? "保存" : "创建"
                }}
              </button>
            </div>
          </form>
        </div>
      </div>

      <div
        *ngIf="showInviteModal"
        class="modal-overlay"
        (click)="closeInviteModal()"
      >
        <div class="modal" (click)="$event.stopPropagation()">
          <div class="modal-header">
            <h2>邀请成员</h2>
            <button class="btn-icon" (click)="closeInviteModal()">
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
          <form [formGroup]="inviteForm" (ngSubmit)="onSubmitInvite()">
            <div class="form-group">
              <label for="inviteEmail">邮箱</label>
              <input
                type="email"
                id="inviteEmail"
                formControlName="email"
                placeholder="输入成员邮箱"
              />
            </div>
            <div class="form-group">
              <label for="inviteRole">角色</label>
              <select id="inviteRole" formControlName="role">
                <option value="member">成员</option>
                <option value="admin">管理员</option>
              </select>
            </div>
            <div class="modal-actions">
              <button
                type="button"
                class="btn btn-secondary"
                (click)="closeInviteModal()"
              >
                取消
              </button>
              <button
                type="submit"
                class="btn btn-primary"
                [disabled]="inviteForm.invalid || submittingInvite"
              >
                {{ submittingInvite ? "邀请中..." : "邀请" }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  `,
  styles: [
    `
      .board-detail-container {
        height: 100vh;
        display: flex;
        flex-direction: column;
        overflow: hidden;
      }

      .board-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 1rem 1.5rem;
        background: var(--bg-primary);
        border-bottom: 1px solid var(--border-color);
      }

      .header-left {
        display: flex;
        align-items: center;
        gap: 1rem;

        h1 {
          margin: 0;
          font-size: 1.25rem;
        }
      }

      .header-actions {
        display: flex;
        align-items: center;
        gap: 0.5rem;
      }

      .board-content {
        flex: 1;
        display: flex;
        overflow: hidden;
      }

      .columns-container {
        flex: 1;
        display: flex;
        overflow-x: auto;
        overflow-y: hidden;
        padding: 1.5rem;
        gap: 1.5rem;
      }

      .column-wrapper {
        flex: 0 0 320px;
      }

      .column {
        background: var(--bg-primary);
        border-radius: var(--radius-md);
        display: flex;
        flex-direction: column;
        height: 100%;
        border: 1px solid var(--border-color);
      }

      .column-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 1rem;
        border-bottom: 1px solid var(--border-color);
        cursor: grab;

        h3 {
          margin: 0;
          font-size: 0.9375rem;
        }
      }

      .column-actions {
        display: flex;
        gap: 0.25rem;
      }

      .cards-container {
        flex: 1;
        overflow-y: auto;
        padding: 0.75rem;
        min-height: 100px;
      }

      .card {
        background: var(--bg-secondary);
        border-radius: var(--radius-sm);
        padding: 0.75rem;
        margin-bottom: 0.75rem;
        cursor: pointer;
        transition: all var(--transition-fast);
        border: 1px solid transparent;

        &:hover {
          border-color: var(--primary-color);
          box-shadow: var(--shadow-sm);
        }

        &:last-child {
          margin-bottom: 0;
        }
      }

      .card-header {
        display: flex;
        align-items: flex-start;
        justify-content: space-between;
        gap: 0.5rem;
        margin-bottom: 0.5rem;

        h4 {
          margin: 0;
          font-size: 0.875rem;
          font-weight: 500;
          word-break: break-word;
        }
      }

      .priority-badge {
        font-size: 0.625rem;
        font-weight: 600;
        padding: 0.125rem 0.375rem;
        border-radius: 3px;
        flex-shrink: 0;

        &.P0 {
          background: #fef2f2;
          color: var(--danger-color);
        }

        &.P1 {
          background: #fff7ed;
          color: var(--warning-color);
        }

        &.P2 {
          background: #f0f9ff;
          color: var(--info-color);
        }

        &.P3 {
          background: var(--bg-tertiary);
          color: var(--text-secondary);
        }
      }

      .card-description {
        margin: 0 0 0.5rem 0;
        font-size: 0.75rem;
        color: var(--text-secondary);
        line-height: 1.4;
        word-break: break-word;
      }

      .card-footer {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-top: 0.5rem;
      }

      .card-due {
        display: flex;
        align-items: center;
        gap: 0.25rem;
        font-size: 0.75rem;
        color: var(--text-secondary);
      }

      .card-assignee .avatar {
        width: 24px;
        height: 24px;
        border-radius: 50%;
        background: var(--primary-color);
        color: white;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 0.75rem;
        font-weight: 500;
      }

      .add-card-btn {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.5rem;
        padding: 0.75rem;
        background: transparent;
        border: none;
        color: var(--text-secondary);
        cursor: pointer;
        font-size: 0.8125rem;
        transition: all var(--transition-fast);
        width: 100%;
        border-radius: 0 0 var(--radius-md) var(--radius-md);

        &:hover {
          background: var(--bg-secondary);
          color: var(--text-primary);
        }
      }

      .empty-columns {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 100%;
        color: var(--text-secondary);
      }

      .side-panel {
        width: 360px;
        background: var(--bg-primary);
        border-left: 1px solid var(--border-color);
        display: none;
        flex-direction: column;
        overflow: hidden;

        &.active {
          display: flex;
        }
      }

      .panel-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 1rem 1.5rem;
        border-bottom: 1px solid var(--border-color);

        h3 {
          margin: 0;
          font-size: 1rem;
        }
      }

      .activity-list,
      .members-list {
        flex: 1;
        overflow-y: auto;
        padding: 1rem;
      }

      .activity-item {
        display: flex;
        gap: 0.75rem;
        padding: 0.75rem 0;
        border-bottom: 1px solid var(--border-color);

        &:last-child {
          border-bottom: none;
        }
      }

      .activity-avatar {
        width: 32px;
        height: 32px;
        border-radius: 50%;
        background: var(--primary-color);
        color: white;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 0.8125rem;
        font-weight: 500;
        flex-shrink: 0;
      }

      .activity-content {
        flex: 1;
        min-width: 0;
      }

      .activity-text {
        margin: 0 0 0.25rem 0;
        font-size: 0.8125rem;
        color: var(--text-primary);
        line-height: 1.4;

        strong {
          color: var(--text-primary);
        }
      }

      .activity-time {
        margin: 0;
        font-size: 0.6875rem;
        color: var(--text-secondary);
      }

      .empty-activities {
        text-align: center;
        padding: 2rem;
        color: var(--text-secondary);
        font-size: 0.875rem;
      }

      .member-item {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        padding: 0.75rem 0;
        border-bottom: 1px solid var(--border-color);

        &:last-child {
          border-bottom: none;
        }
      }

      .member-avatar {
        width: 36px;
        height: 36px;
        border-radius: 50%;
        background: var(--primary-color);
        color: white;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 0.875rem;
        font-weight: 500;
        flex-shrink: 0;
      }

      .member-info {
        flex: 1;
        min-width: 0;
      }

      .member-name {
        margin: 0;
        font-size: 0.875rem;
        font-weight: 500;
      }

      .member-email {
        margin: 0;
        font-size: 0.75rem;
        color: var(--text-secondary);
      }

      .role-badge {
        font-size: 0.625rem;
        font-weight: 600;
        padding: 0.25rem 0.5rem;
        border-radius: var(--radius-sm);
        text-transform: uppercase;

        &.owner {
          background: #fef3c7;
          color: #d97706;
        }

        &.admin {
          background: #dbeafe;
          color: #2563eb;
        }

        &.member {
          background: var(--bg-tertiary);
          color: var(--text-secondary);
        }
      }

      .btn {
        display: inline-flex;
        align-items: center;
        gap: 0.375rem;
        padding: 0.5rem 0.875rem;
        border: none;
        border-radius: var(--radius-md);
        font-size: 0.8125rem;
        font-weight: 500;
        cursor: pointer;
        transition: all var(--transition-fast);

        &:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        &.active {
          background: var(--primary-color);
          color: white;
        }
      }

      .btn-sm {
        padding: 0.375rem 0.75rem;
        font-size: 0.75rem;
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

      .btn-danger {
        background: #fef2f2;
        color: var(--danger-color);

        &:hover:not(:disabled) {
          background: #fee2e2;
        }
      }

      .btn-block {
        width: 100%;
        justify-content: center;
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

        &.large-modal {
          max-width: 600px;
        }
      }

      .modal-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 1.5rem;
        border-bottom: 1px solid var(--border-color);

        h2 {
          margin: 0;
          font-size: 1.125rem;
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
          font-size: 0.8125rem;
        }

        input,
        textarea,
        select {
          width: 100%;
          padding: 0.625rem 0.875rem;
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
          min-height: 100px;
        }

        select {
          cursor: pointer;
        }
      }

      .form-row {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 1rem;
      }

      .color-picker {
        display: flex;
        gap: 0.5rem;
      }

      .color-option {
        width: 28px;
        height: 28px;
        border-radius: 4px;
        cursor: pointer;
        border: 2px solid transparent;
        transition: all var(--transition-fast);

        &.selected {
          border-color: var(--text-primary);
          box-shadow: var(--shadow-sm);
        }
      }

      .modal-actions {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 0.75rem;
        margin-top: 1.5rem;
      }

      .mt-4 {
        margin-top: 1rem;
      }
    `,
  ],
})
export class BoardDetailComponent implements OnInit, OnDestroy {
  boardId: string = "";
  board: Board | null = null;
  columns: ColumnWithCards[] = [];
  tags: Tag[] = [];
  members: BoardMember[] = [];
  activities: ActivityWithUser[] = [];
  loading = false;

  showActivityPanel = false;
  showMembersPanel = false;
  showCreateColumnModal = false;
  showEditColumnModal = false;
  showCreateCardModal = false;
  showInviteModal = false;

  editingColumn: Column | null = null;
  editingCard: Card | null = null;
  currentColumn: Column | null = null;

  columnForm: FormGroup;
  cardForm: FormGroup;
  inviteForm: FormGroup;

  submittingColumn = false;
  submittingCard = false;
  submittingInvite = false;

  colorOptions = [
    "#e2e8f0",
    "#fecaca",
    "#fed7aa",
    "#fde68a",
    "#bbf7d0",
    "#bfdbfe",
    "#ddd6fe",
    "#fbcfe8",
  ];

  private destroy$ = new Subject<void>();

  constructor(
    private route: ActivatedRoute,
    private router: Router,
    private apiService: ApiService,
    private fb: FormBuilder,
  ) {
    this.columnForm = this.fb.group({
      name: ["", [Validators.required, Validators.maxLength(50)]],
      color: ["#e2e8f0"],
    });

    this.cardForm = this.fb.group({
      title: ["", [Validators.required, Validators.maxLength(200)]],
      description: [""],
      priority: ["P3"],
      due_date: [""],
      assignee_id: [null],
    });

    this.inviteForm = this.fb.group({
      email: ["", [Validators.required, Validators.email]],
      role: ["member"],
    });
  }

  ngOnInit(): void {
    this.boardId = this.route.snapshot.params["id"];
    this.loadBoardData();
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }

  get connectedLists(): string[] {
    return this.columns.map((c) => c.id);
  }

  loadBoardData(): void {
    this.loading = true;

    this.apiService.getBoard(this.boardId).subscribe({
      next: (board) => {
        this.board = board;
      },
    });

    this.apiService.getColumns(this.boardId).subscribe({
      next: (columns) => {
        this.columns = columns.map((col) => ({ ...col, cards: [] }));
        this.loadCards();
      },
    });

    this.apiService.getTags(this.boardId).subscribe({
      next: (tags) => {
        this.tags = tags;
      },
    });

    this.apiService.getBoardMembers(this.boardId).subscribe({
      next: (members) => {
        this.members = members;
      },
    });

    this.loadActivities();
  }

  loadCards(): void {
    if (this.columns.length === 0) {
      this.loading = false;
      return;
    }

    const cardRequests = this.columns.map((column) =>
      this.apiService.getCards(column.id),
    );

    forkJoin(cardRequests).subscribe({
      next: (allCards) => {
        allCards.forEach((cards, index) => {
          if (this.columns[index]) {
            this.columns[index].cards = cards;
          }
        });
      },
      error: () => {
        this.loading = false;
      },
      complete: () => {
        this.loading = false;
      },
    });
  }

  loadActivities(): void {
    this.apiService.getActivities(this.boardId, 50).subscribe({
      next: (activities) => {
        this.activities = activities;
      },
    });
  }

  toggleActivityPanel(): void {
    this.showActivityPanel = !this.showActivityPanel;
    this.showMembersPanel = false;
  }

  toggleMembersPanel(): void {
    this.showMembersPanel = !this.showMembersPanel;
    this.showActivityPanel = false;
  }

  addCard(column: Column): void {
    this.currentColumn = column;
    this.editingCard = null;
    this.cardForm.reset({
      title: "",
      description: "",
      priority: "P3",
      due_date: "",
      assignee_id: null,
    });
    this.showCreateCardModal = true;
  }

  openCardDetail(card: Card, column: Column): void {
    this.currentColumn = column;
    this.editingCard = card;
    this.cardForm.patchValue({
      title: card.title,
      description: card.description || "",
      priority: card.priority,
      due_date: card.due_date ? this.formatDateTimeLocal(card.due_date) : "",
      assignee_id: card.assignee_id || null,
    });
    this.showCreateCardModal = true;
  }

  openEditColumn(column: Column): void {
    this.editingColumn = column;
    this.columnForm.patchValue({
      name: column.name,
      color: column.color,
    });
    this.showEditColumnModal = true;
  }

  closeColumnModal(): void {
    this.showCreateColumnModal = false;
    this.showEditColumnModal = false;
    this.editingColumn = null;
    this.columnForm.reset({ name: "", color: "#e2e8f0" });
  }

  closeCardModal(): void {
    this.showCreateCardModal = false;
    this.editingCard = null;
    this.currentColumn = null;
    this.cardForm.reset();
  }

  closeInviteModal(): void {
    this.showInviteModal = false;
    this.inviteForm.reset({ email: "", role: "member" });
  }

  onSubmitColumn(): void {
    if (this.columnForm.invalid) {
      return;
    }

    this.submittingColumn = true;
    const request = this.columnForm.value;

    if (this.showEditColumnModal && this.editingColumn) {
      this.apiService
        .updateColumn(this.editingColumn.id, request)
        .pipe(finalize(() => (this.submittingColumn = false)))
        .subscribe({
          next: () => {
            this.closeColumnModal();
            this.loadBoardData();
          },
        });
    } else {
      this.apiService
        .createColumn(this.boardId, { name: request.name })
        .pipe(finalize(() => (this.submittingColumn = false)))
        .subscribe({
          next: () => {
            this.closeColumnModal();
            this.loadBoardData();
          },
        });
    }
  }

  onSubmitCard(): void {
    if (this.cardForm.invalid || !this.currentColumn) {
      return;
    }

    this.submittingCard = true;
    const formValue = this.cardForm.value;
    const request = {
      title: formValue.title,
      description: formValue.description || null,
      priority: formValue.priority,
      due_date: formValue.due_date || null,
      assignee_id: formValue.assignee_id,
    };

    if (this.editingCard) {
      this.apiService
        .updateCard(this.editingCard.id, request)
        .pipe(finalize(() => (this.submittingCard = false)))
        .subscribe({
          next: () => {
            this.closeCardModal();
            this.loadActivities();
          },
        });
    } else {
      this.apiService
        .createCard(this.currentColumn.id, {
          title: request.title,
          description: request.description,
        })
        .pipe(finalize(() => (this.submittingCard = false)))
        .subscribe({
          next: (card) => {
            const column = this.columns.find(
              (c) => c.id === this.currentColumn!.id,
            );
            if (column) {
              column.cards.push(card);
            }
            this.closeCardModal();
            this.loadActivities();
          },
        });
    }
  }

  deleteCard(): void {
    if (!this.editingCard || !confirm("确定要删除这个卡片吗？")) {
      return;
    }

    this.apiService.deleteCard(this.editingCard.id).subscribe({
      next: () => {
        this.closeCardModal();
        this.loadActivities();
      },
    });
  }

  onSubmitInvite(): void {
    if (this.inviteForm.invalid) {
      return;
    }

    this.submittingInvite = true;

    this.apiService
      .inviteMember(this.boardId, this.inviteForm.value)
      .pipe(finalize(() => (this.submittingInvite = false)))
      .subscribe({
        next: () => {
          this.closeInviteModal();
          this.apiService.getBoardMembers(this.boardId).subscribe({
            next: (members) => {
              this.members = members;
            },
          });
        },
      });
  }

  dropColumn(event: CdkDragDrop<ColumnWithCards[]>): void {
    if (event.previousIndex === event.currentIndex) {
      return;
    }

    moveItemInArray(this.columns, event.previousIndex, event.currentIndex);

    const targetIndex = event.currentIndex;
    const afterColumnId =
      targetIndex > 0 ? this.columns[targetIndex - 1].id : undefined;

    const draggedColumn = event.item.data as Column;

    this.apiService
      .reorderColumn(draggedColumn.id, { after_column_id: afterColumnId })
      .subscribe();
  }

  dropCard(event: CdkDragDrop<Card[]>): void {
    const { card, columnId: prevColumnId } = event.item.data as {
      card: Card;
      columnId: string;
    };

    if (event.previousContainer === event.container) {
      moveItemInArray(
        event.container.data,
        event.previousIndex,
        event.currentIndex,
      );

      const targetIndex = event.currentIndex;
      const afterCardId =
        targetIndex > 0 ? event.container.data[targetIndex - 1].id : undefined;

      this.apiService
        .moveCard(card.id, {
          target_column_id: prevColumnId,
          after_card_id: afterCardId,
        })
        .subscribe();
    } else {
      transferArrayItem(
        event.previousContainer.data,
        event.container.data,
        event.previousIndex,
        event.currentIndex,
      );

      const targetIndex = event.currentIndex;
      const afterCardId =
        targetIndex > 0 ? event.container.data[targetIndex - 1].id : undefined;

      this.apiService
        .moveCard(card.id, {
          target_column_id: event.container.id,
          after_card_id: afterCardId,
        })
        .subscribe();
    }

    this.loadActivities();
  }

  trackByColumn(index: number, column: ColumnWithCards): string {
    return column.id;
  }

  trackByCard(index: number, card: Card): string {
    return card.id;
  }

  getPriorityClass(priority: Priority): string {
    return priority;
  }

  getAvatarInitials(_assigneeId: string): string {
    return "U";
  }

  formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString("zh-CN", {
      month: "short",
      day: "numeric",
    });
  }

  formatDateTimeLocal(dateString: string): string {
    const date = new Date(dateString);
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const day = String(date.getDate()).padStart(2, "0");
    const hours = String(date.getHours()).padStart(2, "0");
    const minutes = String(date.getMinutes()).padStart(2, "0");
    return `${year}-${month}-${day}T${hours}:${minutes}`;
  }

  formatRelativeTime(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diff = now.getTime() - date.getTime();

    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) {
      return `${days} 天前`;
    } else if (hours > 0) {
      return `${hours} 小时前`;
    } else if (minutes > 0) {
      return `${minutes} 分钟前`;
    } else {
      return "刚刚";
    }
  }

  formatActivity(activityWithUser: ActivityWithUser): string {
    const { activity } = activityWithUser;

    const actionMap: Record<string, string> = {
      created: "创建了",
      updated: "更新了",
      deleted: "删除了",
      moved: "移动了",
      invited: "邀请了",
      role_changed: "更改了",
      removed: "移除了",
      tag_added: "为",
      tag_removed: "从",
      checklist_created: "为",
      checklist_deleted: "从",
      checklist_item_completed: "完成了",
      checklist_item_uncompleted: "取消完成了",
    };

    const entityMap: Record<string, string> = {
      board: "看板",
      column: "列",
      card: "卡片",
      tag: "标签",
      member: "成员",
    };

    const action = actionMap[activity.action] || activity.action;
    const entity = entityMap[activity.entity_type] || activity.entity_type;

    return `${action}${entity}`;
  }
}
