import {Injectable} from '@angular/core';


interface Alert {
    type: string;
    message: string;
}

@Injectable({
    providedIn: 'root'
})
export class UiService {

    alerts: Array<Alert> = [];

    constructor() {
    }

    close(alert: Alert) {
        this.alerts.splice(this.alerts.indexOf(alert), 1);
    }

    alert(alert: Alert) {
        this.alerts.push(alert);
    }

    error(message: string) {
        this.alert({type: "danger", message: message});
    }
}
