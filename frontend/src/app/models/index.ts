export interface User {
  id: string;
  username: string;
  email: string;
}

export interface Board {
  id: string;
  name: string;
  description?: string;
  owner_id: string;
  created_at: string;
  updated_at: string;
}

export interface BoardMember {
  id: string;
  board_id: string;
  user_id: string;
  username: string;
  email: string;
  role: BoardRole;
  joined_at: string;
}

export type BoardRole = "owner" | "admin" | "member";

export interface Column {
  id: string;
  board_id: string;
  name: string;
  position: number;
  color: string;
  created_at: string;
  updated_at: string;
}

export interface Card {
  id: string;
  column_id: string;
  title: string;
  description?: string;
  position: number;
  priority: Priority;
  due_date?: string;
  assignee_id?: string;
  created_at: string;
  updated_at: string;
}

export type Priority = "P0" | "P1" | "P2" | "P3";

export interface CardWithDetails {
  card: Card;
  assignee?: User;
  tags: Tag[];
  checklists: ChecklistWithItems[];
}

export interface Tag {
  id: string;
  board_id: string;
  name: string;
  color: string;
  created_at: string;
}

export interface Checklist {
  id: string;
  card_id: string;
  title: string;
  position: number;
  created_at: string;
  updated_at: string;
}

export interface ChecklistItem {
  id: string;
  checklist_id: string;
  content: string;
  is_completed: boolean;
  position: number;
  created_at: string;
  updated_at: string;
}

export interface ChecklistWithItems {
  checklist: Checklist;
  items: ChecklistItem[];
}

export interface Activity {
  id: string;
  board_id: string;
  user_id: string;
  action: string;
  entity_type: EntityType;
  entity_id?: string;
  details: Record<string, unknown>;
  created_at: string;
}

export type EntityType = "board" | "column" | "card" | "tag" | "member";

export interface ActivityWithUser {
  activity: Activity;
  user: User;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  username: string;
  email: string;
  password: string;
}

export interface CreateBoardRequest {
  name: string;
  description?: string;
}

export interface UpdateBoardRequest {
  name?: string;
  description?: string;
}

export interface CreateColumnRequest {
  name: string;
  after_column_id?: string;
}

export interface UpdateColumnRequest {
  name?: string;
  color?: string;
}

export interface ReorderColumnRequest {
  after_column_id?: string;
}

export interface CreateCardRequest {
  title: string;
  description?: string;
  after_card_id?: string;
}

export interface UpdateCardRequest {
  title?: string;
  description?: string;
  priority?: Priority;
  due_date?: string;
  assignee_id?: string;
}

export interface MoveCardRequest {
  target_column_id: string;
  after_card_id?: string;
}

export interface CreateTagRequest {
  name: string;
  color: string;
}

export interface UpdateTagRequest {
  name?: string;
  color?: string;
}

export interface InviteMemberRequest {
  email: string;
  role: BoardRole;
}

export interface UpdateMemberRoleRequest {
  role: BoardRole;
}

export interface CreateChecklistRequest {
  title: string;
}

export interface UpdateChecklistRequest {
  title?: string;
}

export interface CreateChecklistItemRequest {
  content: string;
}

export interface UpdateChecklistItemRequest {
  content?: string;
}
