import {Component, OnInit} from '@angular/core';
import {CryptoService} from "../crypto.service";
import {Router} from "@angular/router";

@Component({
    selector: 'app-page-chat-create',
    templateUrl: './page-chat-create.component.html',
    styleUrls: ['./page-chat-create.component.less']
})
export class PageChatCreateComponent implements OnInit {

    constructor(private crypto: CryptoService, private router: Router) {
    }

    ngOnInit(): void {
    }

    createChat() {
        let channel = this.crypto.generateChannel();
        let keypair = this.crypto.generatePublicPrivateKeys();
        let key = this.crypto.generateKey();
        this.router.navigate(['/chat', channel], {fragment: key + ':' + keypair.secret});
    }

}
