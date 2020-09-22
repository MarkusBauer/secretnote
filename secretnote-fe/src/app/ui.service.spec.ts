import {TestBed} from '@angular/core/testing';

import {UiService} from './ui.service';
import {RouterTestingModule} from "@angular/router/testing";

describe('UiService', () => {
    let service: UiService;

    beforeEach(() => {
        TestBed.configureTestingModule({
            imports: [RouterTestingModule]
        });
        service = TestBed.inject(UiService);
    });

    it('should be created', () => {
        expect(service).toBeTruthy();
    });
});
