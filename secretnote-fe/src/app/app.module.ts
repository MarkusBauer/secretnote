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

@NgModule({
    declarations: [
        AppComponent,
        PageNoteStoreComponent,
        PageNoteRetrieveComponent,
        PageFaqComponent,
    ],
    imports: [
        BrowserModule,
        FormsModule,
        HttpClientModule,
        AppRoutingModule,
        NgbModule
    ],
    providers: [],
    bootstrap: [AppComponent]
})
export class AppModule {
}
