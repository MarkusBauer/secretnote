import {Component, OnDestroy, OnInit} from '@angular/core';
import {ActivatedRoute} from "@angular/router";
import {BackendService} from "../backend.service";
import {WebSocketSubject} from "rxjs/internal-compatibility";
import {ChatMessage, CryptoService} from "../crypto.service";
import {UiService} from "../ui.service";
import {Subscription} from "rxjs";
import {UsernamesService, UserInfo} from "../usernames.service";


interface ExtendedChatMessage extends ChatMessage {
    senderInfo: UserInfo;
}


@Component({
    selector: 'app-page-chat',
    templateUrl: './page-chat.component.html',
    styleUrls: ['./page-chat.component.less']
})
export class PageChatComponent implements OnInit, OnDestroy {

    channel: string;
    key: string;
    userPrivate: string;
    userPublic: string;
    connection: WebSocketSubject<ArrayBuffer>;
    subscription: Subscription;
    timeout: number = null;
    connected: boolean = false;

    privateUrl: string;
    publicUrl: string;
    me: UserInfo;

    messages: Array<ExtendedChatMessage> = [];
    knownMessageSize = null;
    loadedMessages = null;
    loadMoreMessagesSubscription: Subscription;

    textinput: string = '';

    constructor(private route: ActivatedRoute,
                private backend: BackendService,
                private crypto: CryptoService,
                private ui: UiService,
                private usernames: UsernamesService) {
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
    }

    ngOnDestroy() {
        if (this.timeout !== null) {
            clearTimeout(this.timeout);
            this.timeout = null;
        }
        if (this.subscription) {
            this.subscription.unsubscribe();
            this.subscription = null;
        }
    }

    updateChannelKey() {
        if (!this.channel || !this.key) return;
        if (this.channel.length != 24) {
            // TODO report error
            this.ui.error("Invalid channel ID");
            return
        }
        // TODO check key
        if (!this.crypto.isValidKey(this.key)) {
            this.ui.error("Invalid key");
            return;
        }
        if (!this.userPrivate || !this.userPublic) {
            this.ui.error("Invalid user key");
            return
        }

        this.publicUrl = this.backend.generateChatPublicUrl(this.channel, this.key);
        this.privateUrl = this.backend.generateChatPrivateUrl(this.channel, this.key, this.userPrivate);
        this.me = this.usernames.getUserInfo(this.userPublic, this.channel);

        if (this.connection && this.subscription) {
            this.connection.complete();
            this.subscription.unsubscribe();
            this.connection = null;
            this.subscription = null;
            this.messages = [];
            this.knownMessageSize = null;
            this.loadedMessages = null;
        }
        this.connection = this.backend.connectToChat(this.channel, (e) => {
            this.connected = true;
        }, (e) => {
            this.connected = false;
        });
        this.subscribeWebsocket();
        this.loadMoreMessages();
    }

    private subscribeWebsocket() {
        this.timeout = null;
        this.subscription = this.connection.subscribe(data => {
            this.messages.push(this.readMessage(data));
        }, err => {
            console.log('ERROR', err);
            this.timeout = setTimeout(() => {
                this.subscribeWebsocket()
            }, 3000);
        });
    }

    sendMessage(text: string) {
        text = text.trim();
        if (!text) return;
        let msg: ChatMessage = {sender: this.userPublic, ts: new Date().getTime(), text: text};
        let bin = this.crypto.encryptChatMessage(msg, this.key, this.userPrivate);
        this.connection.next(bin);
        this.textinput = '';
    }

    readMessage(data: string | ArrayBuffer): ExtendedChatMessage {
        try {
            let msg = this.crypto.decryptChatMessage(data, this.key) as ExtendedChatMessage;
            msg.senderInfo = this.usernames.getUserInfo(msg.sender, this.channel);
            return msg;
        } catch (e) {
            return {
                sender: "(system)",
                text: "Invalid message: " + e,
                ts: Date.now(),
                senderInfo: new UserInfo("(system)", 0, "", "#aaaaaa", true)
            };
        }
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
                newMessages.push(this.readMessage(result.messages[i]));
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

    onMessagetextKeydown($event: KeyboardEvent) {
        if ($event.key === "Enter" && !$event.shiftKey) {
            $event.preventDefault();
            this.sendMessage(this.textinput);
        }
    }
}
