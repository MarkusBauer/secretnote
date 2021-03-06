import {NgModule} from '@angular/core';
import {Routes, RouterModule} from '@angular/router';
import {PageFaqComponent} from "./page-faq/page-faq.component";
import {PageNoteStoreComponent} from "./page-note-store/page-note-store.component";
import {PageNoteRetrieveComponent} from "./page-note-retrieve/page-note-retrieve.component";
import {PageNoteAdminComponent} from "./page-note-admin/page-note-admin.component";
import {PageChatComponent} from "./page-chat/page-chat.component";
import {PageChatCreateComponent} from "./page-chat-create/page-chat-create.component";
import {ChatJoinGuard} from "./chat-join.guard";
import {PageAboutComponent} from "./page-about/page-about.component";
import {PageNotFoundComponent} from "./page-not-found/page-not-found.component";

const routes: Routes = [
    {path: '', component: PageNoteStoreComponent},
    {path: 'note/store', component: PageNoteStoreComponent},
    {path: 'note/admin/:ident', component: PageNoteAdminComponent},
    {path: 'note/:ident', component: PageNoteRetrieveComponent},
    {path: 'chat/create', component: PageChatCreateComponent},
    {path: 'chat/join/:channel', children: [], canActivate: [ChatJoinGuard]},
    {path: 'chat/:channel', component: PageChatComponent},
    {path: 'faq', component: PageFaqComponent},
    {path: 'about', component: PageAboutComponent},
    {path: '**', component: PageNotFoundComponent}
];

@NgModule({
    imports: [RouterModule.forRoot(routes, {
    initialNavigation: 'enabled',
    relativeLinkResolution: 'legacy'
})],
    exports: [RouterModule]
})
export class AppRoutingModule {
}
