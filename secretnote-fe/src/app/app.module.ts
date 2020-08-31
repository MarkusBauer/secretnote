import {BrowserModule} from '@angular/platform-browser';
import {NgModule} from '@angular/core';

import {AppRoutingModule} from './app-routing.module';
import {AppComponent} from './app.component';
import {NgbModule} from '@ng-bootstrap/ng-bootstrap';
import {PageNoteStoreComponent} from './page-note-store/page-note-store.component';
import {PageNoteRetrieveComponent} from './page-note-retrieve/page-note-retrieve.component';
import {PageFaqComponent} from './page-faq/page-faq.component';
import {HttpClientModule} from "@angular/common/http";
import {FormsModule} from "@angular/forms";
import { PageNoteAdminComponent } from './page-note-admin/page-note-admin.component';
import { PageChatComponent } from './page-chat/page-chat.component';

@NgModule({
    declarations: [
        AppComponent,
        PageNoteStoreComponent,
        PageNoteRetrieveComponent,
        PageFaqComponent,
        PageNoteAdminComponent,
        PageChatComponent,
    ],
    imports: [
        BrowserModule,
        FormsModule,
        HttpClientModule,
        AppRoutingModule,
        NgbModule,
    ],
    providers: [],
    bootstrap: [AppComponent]
})
export class AppModule {
}
