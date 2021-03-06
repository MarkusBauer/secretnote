import {BrowserModule} from '@angular/platform-browser';
import {NgModule} from '@angular/core';

import {AppRoutingModule} from './app-routing.module';
import {AppComponent} from './app.component';
import {NgbCollapseModule, NgbDropdownModule, NgbToastModule, NgbTooltipModule} from '@ng-bootstrap/ng-bootstrap';
import {PageNoteStoreComponent} from './page-note-store/page-note-store.component';
import {PageNoteRetrieveComponent} from './page-note-retrieve/page-note-retrieve.component';
import {PageFaqComponent} from './page-faq/page-faq.component';
import {HttpClientModule} from "@angular/common/http";
import {FormsModule} from "@angular/forms";
import { PageNoteAdminComponent } from './page-note-admin/page-note-admin.component';
import { PageChatComponent } from './page-chat/page-chat.component';
import { PageChatCreateComponent } from './page-chat-create/page-chat-create.component';
import { PageAboutComponent } from './page-about/page-about.component';
import { OcticonDirective } from './octicon.directive';
import { PageNotFoundComponent } from './page-not-found/page-not-found.component';

@NgModule({
    declarations: [
        AppComponent,
        PageNoteStoreComponent,
        PageNoteRetrieveComponent,
        PageFaqComponent,
        PageNoteAdminComponent,
        PageChatComponent,
        PageChatCreateComponent,
        PageAboutComponent,
        OcticonDirective,
        PageNotFoundComponent,
    ],
    imports: [
        BrowserModule.withServerTransition({ appId: 'serverApp' }),
        FormsModule,
        HttpClientModule,
        AppRoutingModule,
        NgbTooltipModule,
        NgbCollapseModule,
        NgbToastModule,
        NgbDropdownModule,
    ],
    providers: [],
    bootstrap: [AppComponent]
})
export class AppModule {
}
