import { Routes } from "@angular/router";
import { AuthGuard, UnauthGuard } from "./guards/auth.guard";

export const routes: Routes = [
  {
    path: "",
    redirectTo: "/boards",
    pathMatch: "full",
  },
  {
    path: "login",
    loadComponent: () =>
      import("./components/login/login.component").then(
        (m) => m.LoginComponent,
      ),
    canActivate: [UnauthGuard],
  },
  {
    path: "register",
    loadComponent: () =>
      import("./components/register/register.component").then(
        (m) => m.RegisterComponent,
      ),
    canActivate: [UnauthGuard],
  },
  {
    path: "boards",
    loadComponent: () =>
      import("./components/board-list/board-list.component").then(
        (m) => m.BoardListComponent,
      ),
    canActivate: [AuthGuard],
  },
  {
    path: "boards/:id",
    loadComponent: () =>
      import("./components/board-detail/board-detail.component").then(
        (m) => m.BoardDetailComponent,
      ),
    canActivate: [AuthGuard],
  },
  {
    path: "**",
    redirectTo: "/boards",
  },
];
