import { Injectable } from "@angular/core";
import { HttpClient, HttpHeaders } from "@angular/common/http";
import { BehaviorSubject, Observable, tap } from "rxjs";
import { User, AuthResponse, LoginRequest, RegisterRequest } from "../models";

const AUTH_TOKEN_KEY = "kanban_auth_token";
const AUTH_USER_KEY = "kanban_auth_user";

@Injectable({
  providedIn: "root",
})
export class AuthService {
  private readonly apiUrl = "/api/auth";
  private currentUserSubject = new BehaviorSubject<User | null>(null);
  private tokenSubject = new BehaviorSubject<string | null>(null);

  public currentUser$ = this.currentUserSubject.asObservable();
  public token$ = this.tokenSubject.asObservable();

  constructor(private http: HttpClient) {
    this.initFromStorage();
  }

  private initFromStorage(): void {
    const token = localStorage.getItem(AUTH_TOKEN_KEY);
    const userStr = localStorage.getItem(AUTH_USER_KEY);

    if (token && userStr) {
      try {
        const user = JSON.parse(userStr) as User;
        this.tokenSubject.next(token);
        this.currentUserSubject.next(user);
      } catch {
        this.clearStorage();
      }
    }
  }

  private clearStorage(): void {
    localStorage.removeItem(AUTH_TOKEN_KEY);
    localStorage.removeItem(AUTH_USER_KEY);
    this.tokenSubject.next(null);
    this.currentUserSubject.next(null);
  }

  getToken(): string | null {
    return this.tokenSubject.value;
  }

  getCurrentUser(): User | null {
    return this.currentUserSubject.value;
  }

  isAuthenticated(): boolean {
    return !!this.tokenSubject.value;
  }

  register(request: RegisterRequest): Observable<AuthResponse> {
    return this.http
      .post<AuthResponse>(`${this.apiUrl}/register`, request)
      .pipe(tap((response) => this.handleAuthResponse(response)));
  }

  login(request: LoginRequest): Observable<AuthResponse> {
    return this.http
      .post<AuthResponse>(`${this.apiUrl}/login`, request)
      .pipe(tap((response) => this.handleAuthResponse(response)));
  }

  logout(): void {
    this.clearStorage();
  }

  getCurrentUserFromApi(): Observable<User> {
    return this.http.get<User>(`${this.apiUrl}/me`).pipe(
      tap((user) => {
        this.currentUserSubject.next(user);
        localStorage.setItem(AUTH_USER_KEY, JSON.stringify(user));
      }),
    );
  }

  private handleAuthResponse(response: AuthResponse): void {
    const { token, user } = response;

    localStorage.setItem(AUTH_TOKEN_KEY, token);
    localStorage.setItem(AUTH_USER_KEY, JSON.stringify(user));

    this.tokenSubject.next(token);
    this.currentUserSubject.next(user);
  }

  getAuthHeaders(): HttpHeaders {
    const token = this.getToken();
    if (token) {
      return new HttpHeaders({
        Authorization: `Bearer ${token}`,
      });
    }
    return new HttpHeaders();
  }
}
