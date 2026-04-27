import { Injectable } from "@angular/core";
import { HttpClient } from "@angular/common/http";
import { Observable } from "rxjs";
import { AuthService } from "./auth.service";
import {
  Board,
  CreateBoardRequest,
  UpdateBoardRequest,
  Column,
  CreateColumnRequest,
  UpdateColumnRequest,
  ReorderColumnRequest,
  Card,
  CardWithDetails,
  CreateCardRequest,
  UpdateCardRequest,
  MoveCardRequest,
  Tag,
  CreateTagRequest,
  UpdateTagRequest,
  BoardMember,
  InviteMemberRequest,
  UpdateMemberRoleRequest,
  ActivityWithUser,
  Checklist,
  ChecklistItem,
  CreateChecklistRequest,
  UpdateChecklistRequest,
  CreateChecklistItemRequest,
  UpdateChecklistItemRequest,
} from "../models";

@Injectable({
  providedIn: "root",
})
export class ApiService {
  private readonly apiUrl = "/api";

  constructor(
    private http: HttpClient,
    private authService: AuthService,
  ) {}

  private getHeaders() {
    return {
      headers: this.authService.getAuthHeaders(),
    };
  }

  // Boards
  getBoards(search?: string): Observable<Board[]> {
    let url = `${this.apiUrl}/boards`;
    if (search) {
      url += `?search=${encodeURIComponent(search)}`;
    }
    return this.http.get<Board[]>(url, this.getHeaders());
  }

  getBoard(boardId: string): Observable<Board> {
    return this.http.get<Board>(
      `${this.apiUrl}/boards/${boardId}`,
      this.getHeaders(),
    );
  }

  createBoard(request: CreateBoardRequest): Observable<Board> {
    return this.http.post<Board>(
      `${this.apiUrl}/boards`,
      request,
      this.getHeaders(),
    );
  }

  updateBoard(boardId: string, request: UpdateBoardRequest): Observable<Board> {
    return this.http.put<Board>(
      `${this.apiUrl}/boards/${boardId}`,
      request,
      this.getHeaders(),
    );
  }

  deleteBoard(boardId: string): Observable<void> {
    return this.http.delete<void>(
      `${this.apiUrl}/boards/${boardId}`,
      this.getHeaders(),
    );
  }

  // Columns
  getColumns(boardId: string): Observable<Column[]> {
    return this.http.get<Column[]>(
      `${this.apiUrl}/boards/${boardId}/columns`,
      this.getHeaders(),
    );
  }

  createColumn(
    boardId: string,
    request: CreateColumnRequest,
  ): Observable<Column> {
    return this.http.post<Column>(
      `${this.apiUrl}/boards/${boardId}/columns`,
      request,
      this.getHeaders(),
    );
  }

  updateColumn(
    columnId: string,
    request: UpdateColumnRequest,
  ): Observable<Column> {
    return this.http.put<Column>(
      `${this.apiUrl}/columns/${columnId}`,
      request,
      this.getHeaders(),
    );
  }

  deleteColumn(columnId: string): Observable<void> {
    return this.http.delete<void>(
      `${this.apiUrl}/columns/${columnId}`,
      this.getHeaders(),
    );
  }

  reorderColumn(
    columnId: string,
    request: ReorderColumnRequest,
  ): Observable<Column> {
    return this.http.put<Column>(
      `${this.apiUrl}/columns/${columnId}/reorder`,
      request,
      this.getHeaders(),
    );
  }

  // Cards
  getCards(columnId: string): Observable<Card[]> {
    return this.http.get<Card[]>(
      `${this.apiUrl}/columns/${columnId}/cards`,
      this.getHeaders(),
    );
  }

  createCard(columnId: string, request: CreateCardRequest): Observable<Card> {
    return this.http.post<Card>(
      `${this.apiUrl}/columns/${columnId}/cards`,
      request,
      this.getHeaders(),
    );
  }

  getCard(cardId: string): Observable<CardWithDetails> {
    return this.http.get<CardWithDetails>(
      `${this.apiUrl}/cards/${cardId}`,
      this.getHeaders(),
    );
  }

  updateCard(cardId: string, request: UpdateCardRequest): Observable<Card> {
    return this.http.put<Card>(
      `${this.apiUrl}/cards/${cardId}`,
      request,
      this.getHeaders(),
    );
  }

  deleteCard(cardId: string): Observable<void> {
    return this.http.delete<void>(
      `${this.apiUrl}/cards/${cardId}`,
      this.getHeaders(),
    );
  }

  moveCard(cardId: string, request: MoveCardRequest): Observable<Card> {
    return this.http.put<Card>(
      `${this.apiUrl}/cards/${cardId}/move`,
      request,
      this.getHeaders(),
    );
  }

  // Tags
  getTags(boardId: string): Observable<Tag[]> {
    return this.http.get<Tag[]>(
      `${this.apiUrl}/boards/${boardId}/tags`,
      this.getHeaders(),
    );
  }

  createTag(boardId: string, request: CreateTagRequest): Observable<Tag> {
    return this.http.post<Tag>(
      `${this.apiUrl}/boards/${boardId}/tags`,
      request,
      this.getHeaders(),
    );
  }

  updateTag(tagId: string, request: UpdateTagRequest): Observable<Tag> {
    return this.http.put<Tag>(
      `${this.apiUrl}/tags/${tagId}`,
      request,
      this.getHeaders(),
    );
  }

  deleteTag(tagId: string): Observable<void> {
    return this.http.delete<void>(
      `${this.apiUrl}/tags/${tagId}`,
      this.getHeaders(),
    );
  }

  addTagToCard(cardId: string, tagId: string): Observable<void> {
    return this.http.post<void>(
      `${this.apiUrl}/cards/${cardId}/tags/${tagId}`,
      {},
      this.getHeaders(),
    );
  }

  removeTagFromCard(cardId: string, tagId: string): Observable<void> {
    return this.http.delete<void>(
      `${this.apiUrl}/cards/${cardId}/tags/${tagId}`,
      this.getHeaders(),
    );
  }

  // Board Members
  getBoardMembers(boardId: string): Observable<BoardMember[]> {
    return this.http.get<BoardMember[]>(
      `${this.apiUrl}/boards/${boardId}/members`,
      this.getHeaders(),
    );
  }

  inviteMember(
    boardId: string,
    request: InviteMemberRequest,
  ): Observable<BoardMember> {
    return this.http.post<BoardMember>(
      `${this.apiUrl}/boards/${boardId}/members`,
      request,
      this.getHeaders(),
    );
  }

  updateMemberRole(
    boardId: string,
    userId: string,
    request: UpdateMemberRoleRequest,
  ): Observable<BoardMember> {
    return this.http.put<BoardMember>(
      `${this.apiUrl}/boards/${boardId}/members/${userId}`,
      request,
      this.getHeaders(),
    );
  }

  removeMember(boardId: string, userId: string): Observable<void> {
    return this.http.delete<void>(
      `${this.apiUrl}/boards/${boardId}/members/${userId}`,
      this.getHeaders(),
    );
  }

  // Activities
  getActivities(
    boardId: string,
    limit?: number,
  ): Observable<ActivityWithUser[]> {
    let url = `${this.apiUrl}/boards/${boardId}/activities`;
    if (limit) {
      url += `?limit=${limit}`;
    }
    return this.http.get<ActivityWithUser[]>(url, this.getHeaders());
  }

  // Checklists
  createChecklist(
    cardId: string,
    request: CreateChecklistRequest,
  ): Observable<Checklist> {
    return this.http.post<Checklist>(
      `${this.apiUrl}/cards/${cardId}/checklists`,
      request,
      this.getHeaders(),
    );
  }

  updateChecklist(
    checklistId: string,
    request: UpdateChecklistRequest,
  ): Observable<Checklist> {
    return this.http.put<Checklist>(
      `${this.apiUrl}/checklists/${checklistId}`,
      request,
      this.getHeaders(),
    );
  }

  deleteChecklist(checklistId: string): Observable<void> {
    return this.http.delete<void>(
      `${this.apiUrl}/checklists/${checklistId}`,
      this.getHeaders(),
    );
  }

  createChecklistItem(
    checklistId: string,
    request: CreateChecklistItemRequest,
  ): Observable<ChecklistItem> {
    return this.http.post<ChecklistItem>(
      `${this.apiUrl}/checklists/${checklistId}/items`,
      request,
      this.getHeaders(),
    );
  }

  updateChecklistItem(
    itemId: string,
    request: UpdateChecklistItemRequest,
  ): Observable<ChecklistItem> {
    return this.http.put<ChecklistItem>(
      `${this.apiUrl}/checklist_items/${itemId}`,
      request,
      this.getHeaders(),
    );
  }

  deleteChecklistItem(itemId: string): Observable<void> {
    return this.http.delete<void>(
      `${this.apiUrl}/checklist_items/${itemId}`,
      this.getHeaders(),
    );
  }

  toggleChecklistItem(itemId: string): Observable<ChecklistItem> {
    return this.http.put<ChecklistItem>(
      `${this.apiUrl}/checklist_items/${itemId}/toggle`,
      {},
      this.getHeaders(),
    );
  }
}
