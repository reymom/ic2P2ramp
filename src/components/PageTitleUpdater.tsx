import { useEffect } from 'react';
import { useLocation } from 'react-router-dom';

const PageTitleUpdater = () => {
    const location = useLocation();

    useEffect(() => {
        switch (location.pathname) {
            case "/create":
                document.title = "icRamp | Create Order";
                break;
            case "/view":
                document.title = "icRamp | View Orders";
                break;
            case "/profile":
                document.title = "icRamp | User Profile";
                break;
            case "/register":
                document.title = "icRamp | Register";
                break;
            case "/confirm-email":
                document.title = "icRamp | Email Confirmation";
                break;
            case "/reset-password":
                document.title = "icRamp | Change Password";
                break;
            case "/forgot-password":
                document.title = "icRamp | Recover Password";
                break;
            case "/":
                document.title = "icRamp | Login";
                break;
            default:
                document.title = "icRamp";
        }
    }, [location.pathname]);

    return null;
};

export default PageTitleUpdater;