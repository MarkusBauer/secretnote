import {NgModule} from '@angular/core';
import {Routes, RouterModule} from '@angular/router';
import {PageFaqComponent} from "./page-faq/page-faq.component";
import {PageNoteStoreComponent} from "./page-note-store/page-note-store.component";
import {PageNoteRetrieveComponent} from "./page-note-retrieve/page-note-retrieve.component";
import {PageNoteAdminComponent} from "./page-note-admin/page-note-admin.component";

const routes: Routes = [
    {path: '', component: PageNoteStoreComponent},
    {path: 'note/admin/:ident', component: PageNoteAdminComponent},
    {path: 'note/:ident', component: PageNoteRetrieveComponent},
    {path: 'faq', component: PageFaqComponent},
];

@NgModule({
    imports: [RouterModule.forRoot(routes)],
    exports: [RouterModule]
})
export class AppRoutingModule {
}
