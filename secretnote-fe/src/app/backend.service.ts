import {Injectable} from '@angular/core';
import {HttpClient} from "@angular/common/http";
import {Observable} from "rxjs";
import {map} from "rxjs/operators";

interface NoteStoreResponse {
    ident: string
}

@Injectable({
    providedIn: 'root'
})
export class BackendService {

    base = '/';

    constructor(private http: HttpClient) {
    }


    storeNote(text: string): Observable<string> {
        return this.http.post<NoteStoreResponse>(this.base + 'api/note/store', {data: text})
            .pipe(map(response => response.ident));
    }

}
