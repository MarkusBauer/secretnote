import {Component, OnInit} from '@angular/core';
import {ActivatedRoute} from "@angular/router";
import {BackendService} from "../backend.service";
import {WebSocketSubject} from "rxjs/internal-compatibility";
import {ChatMessage, CryptoService} from "../crypto.service";
import {UiService} from "../ui.service";
import {Subscription} from "rxjs";


@Component({
    selector: 'app-page-chat',
    templateUrl: './page-chat.component.html',
    styleUrls: ['./page-chat.component.less']
})
export class PageChatComponent implements OnInit {

    channel: string;
    key: string;
    userPrivate: string;
    userPublic: string;
    connection: WebSocketSubject<any>;

    privateUrl: string;
    publicUrl: string;

    messages: Array<ChatMessage> = [];
    knownMessageSize = null;
    loadedMessages = null;
    loadMoreMessagesSubscription: Subscription;

    textinput: string = '';

    constructor(private route: ActivatedRoute,
                private backend: BackendService,
                private crypto: CryptoService,
                private ui: UiService) {
    }

    ngOnInit(): void {
        this.route.paramMap.subscribe(map => {
            this.channel = map.get("channel");
            this.updateChannelKey();
        });
        this.route.fragment.subscribe(f => {
            if (!f || !f.includes(":")) {
                this.ui.error("Invalid key given!");
                return;
            }
            let k = f.split(":");
            this.key = k[0];
            this.userPrivate = k[1];
            this.userPublic = this.crypto.getPublicFromSecretKey(this.userPrivate);
            this.updateChannelKey();
        });
        this.messages = [];
        // TODO remove that later!
        // this.crypto.test();
    }

    updateChannelKey() {
        if (!this.channel || !this.key) return;
        if (this.channel.length != 24) {
            // TODO report error
            this.ui.error("Invalid channel ID");
            return
        }
        // TODO check key
        if (!this.userPrivate || !this.userPublic) {
            this.ui.error("Invalid key");
            return
        }

        this.publicUrl = this.backend.generateChatPublicUrl(this.channel, this.key);
        this.privateUrl = this.backend.generateChatPrivateUrl(this.channel, this.key, this.userPrivate);

        this.connection = this.backend.connectToChat(this.channel);
        this.connection.subscribe(data => {
            try {
                let msg = this.crypto.decryptChatMessage(data, this.key);
                this.messages.push(msg);
            } catch (e) {
                this.messages.push({sender: "(system)", text: "Invalid message: " + e, ts: Date.now()});
            }
        });
        this.loadMoreMessages();
    }

    sendMessage(text: string) {
        text = text.trim();
        if (!text) return;
        let msg: ChatMessage = {sender: this.userPublic, ts: new Date().getTime(), text: text};
        let bin = this.crypto.encryptChatMessage(msg, this.key, this.userPrivate);
        this.connection.next(bin);
        this.textinput = '';
    }

    usernameFromSender(sender: string): string {
        return sender.substring(0, 43);
    }

    formattedDate(ts: number): string {
        let d = new Date(ts);
        const today = new Date();
        if (d.getDate() == today.getDate() && d.getMonth() == today.getMonth() && d.getFullYear() == today.getFullYear()) {
            return d.toLocaleTimeString();
        } else {
            return d.toLocaleString();
        }
    }

    loadMoreMessages() {
        let m = this.backend.getChatMessages(this.channel, this.loadedMessages || 0, this.knownMessageSize || 0, 25);
        this.loadMoreMessagesSubscription = m.subscribe((result) => {
            if (!this.knownMessageSize) this.knownMessageSize = result.len;
            let newMessages = [];
            for (let i = result.messages.length - 1; i >= 0; i--) {
                try {
                    let msg = this.crypto.decryptChatMessage(result.messages[i], this.key);
                    newMessages.push(msg);
                } catch (e) {
                    newMessages.push({sender: "(system)", text: "Invalid message: " + e, ts: Date.now()});
                }
            }
            this.messages = newMessages.concat(this.messages);
            this.loadedMessages += result.messages.length;
            this.loadMoreMessagesSubscription = null;
        }, err => {
            console.error(err);
            this.ui.error('Error receiving messages: ' + err);
            this.loadMoreMessagesSubscription = null;
        });
    }
}
