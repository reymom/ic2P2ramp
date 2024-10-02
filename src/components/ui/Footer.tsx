import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faTelegram, faTwitter, faGithub } from "@fortawesome/free-brands-svg-icons";

const Footer = () => {
    return (
        <footer className="w-full bg-gray-100 py-4 px-6 border-t">
            <div className="flex justify-between items-center">
                <span className="text-gray-700 text-sm">
                    &copy; 2024 - icRamp.
                </span>
                <div className="flex space-x-4 text-gray-700">
                    <a href="https://t.me/+1qd_xreS_hpkMTBk" target="_blank" rel="noopener noreferrer">
                        <FontAwesomeIcon icon={faTelegram} size="lg" />
                    </a>
                    <a href="https://x.com/ic_rampXYZ?t=kjzM0v-CJiSfGR_RC8qSCg&s=09" target="_blank" rel="noopener noreferrer">
                        <FontAwesomeIcon icon={faTwitter} size="lg" />
                    </a>
                    <a href="https://github.com/reymom/ic2P2ramp" target="_blank" rel="noopener noreferrer">
                        <FontAwesomeIcon icon={faGithub} size="lg" />
                    </a>
                </div>
            </div>
        </footer>
    );
};

export default Footer;