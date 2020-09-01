import {Injectable} from '@angular/core';
import {CanActivate, ActivatedRouteSnapshot, RouterStateSnapshot, UrlTree, Router} from '@angular/router';
import {Observable} from 'rxjs';
import {CryptoService} from "./crypto.service";
import {UiService} from "./ui.service";
import {BackendService} from "./backend.service";

@Injectable({
    providedIn: 'root'
})
export class ChatJoinGuard implements CanActivate {

    constructor(private crypto: CryptoService, private ui: UiService, private backend: BackendService, private router: Router) {
    }

    canActivate(next: ActivatedRouteSnapshot,
                state: RouterStateSnapshot): Observable<boolean | UrlTree> | Promise<boolean | UrlTree> | boolean | UrlTree {
        let channel = next.paramMap.get('channel');
        let key = next.fragment;
        if (!this.crypto.isValidKey(key)) {
            this.ui.error("Invalid key: " + JSON.stringify(key));
            return false;
        }
        let keypair = this.crypto.generatePublicPrivateKeys();
        console.log(next, state);
        return this.router.createUrlTree(['chat', channel], {fragment: key + ':' + keypair.secret});
    }

}
