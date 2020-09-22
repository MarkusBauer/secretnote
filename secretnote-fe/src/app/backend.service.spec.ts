import {TestBed} from '@angular/core/testing';

import {BackendService} from './backend.service';
import {HttpClientTestingModule, HttpTestingController} from "@angular/common/http/testing";
import {HttpClient} from "@angular/common/http";
import {RouterTestingModule} from "@angular/router/testing";

describe('BackendService', () => {
    let service: BackendService;
    let httpCtrl: HttpTestingController;

    beforeEach(() => {
        TestBed.configureTestingModule({
            imports: [HttpClientTestingModule, RouterTestingModule],
            providers: [BackendService]
        });
        service = TestBed.inject(BackendService);
        httpCtrl = TestBed.inject(HttpTestingController);
    });

    it('should be created', () => {
        expect(service).toBeTruthy();
    });
});
