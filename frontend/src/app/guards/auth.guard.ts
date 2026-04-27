import { Injectable } from "@angular/core";
import {
  CanActivate,
  ActivatedRouteSnapshot,
  RouterStateSnapshot,
  Router,
  CanActivateChild,
  UrlTree,
} from "@angular/router";
import { Observable, of, switchMap, take } from "rxjs";
import { AuthService } from "../services/auth.service";
import { map } from "rxjs/operators";

@Injectable({
  providedIn: "root",
})
export class AuthGuard implements CanActivate, CanActivateChild {
  constructor(
    private authService: AuthService,
    private router: Router,
  ) {}

  canActivate(
    route: ActivatedRouteSnapshot,
    state: RouterStateSnapshot,
  ): Observable<boolean | UrlTree> {
    return this.checkAuth(state.url);
  }

  canActivateChild(
    childRoute: ActivatedRouteSnapshot,
    state: RouterStateSnapshot,
  ): Observable<boolean | UrlTree> {
    return this.checkAuth(state.url);
  }

  private checkAuth(redirectUrl: string): Observable<boolean | UrlTree> {
    return this.authService.currentUser$.pipe(
      take(1),
      map((user) => {
        if (user) {
          return true;
        }

        return this.router.createUrlTree(["/login"], {
          queryParams: { redirect: redirectUrl },
        });
      }),
    );
  }
}

@Injectable({
  providedIn: "root",
})
export class UnauthGuard implements CanActivate {
  constructor(
    private authService: AuthService,
    private router: Router,
  ) {}

  canActivate(): Observable<boolean | UrlTree> {
    return this.authService.currentUser$.pipe(
      take(1),
      map((user) => {
        if (!user) {
          return true;
        }

        return this.router.createUrlTree(["/boards"]);
      }),
    );
  }
}
