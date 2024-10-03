import { FontAwesomeIcon } from "@fortawesome/react-fontawesome";
import { faTelegram, faTwitter, faGithub, } from "@fortawesome/free-brands-svg-icons";
import { faEnvelope } from "@fortawesome/free-solid-svg-icons";
import notionIcon from "../../assets/notion-icon.png";
import poweredByICP from "../../assets/powered-by-icp.svg";
import icpLogo from "../../assets/blockchains/icp-logo.svg";

const Footer = () => {
    return (
        <footer className="w-full bg-gray-100 py-6 px-10 border-t relative">
            <div className="flex justify-between items-center">
                <span className="text-gray-700 text-sm">
                    &copy; 2024 - icRamp
                </span>
                <div className="absolute inset-0 flex justify-center items-center">
                    <img src={poweredByICP} alt="Powered by ICP" className="hidden sm:block h-4" />
                    {/* Small screen: ICP logo */}
                    <img src={icpLogo} alt="ICP Logo" className="block sm:hidden h-4" />
                </div>
                <div className="flex space-x-4 text-gray-700">
                    <a href="https://mesquite-structure-f75.notion.site/Onboarding-114aa21f9dd480ffb6a0ed741dddc80c" target="_blank" rel="noopener noreferrer">
                        <img src={notionIcon} alt="Notion Logo" title="Onboarding Docs" className="h-6 w-6" />
                    </a>
                    <a href="https://t.me/+1qd_xreS_hpkMTBk" target="_blank" rel="noopener noreferrer">
                        <FontAwesomeIcon icon={faTelegram} size="lg" color="#24A1DE" />
                    </a>
                    <a href="https://x.com/ic_rampXYZ?t=kjzM0v-CJiSfGR_RC8qSCg&s=09" target="_blank" rel="noopener noreferrer">
                        <FontAwesomeIcon icon={faTwitter} size="lg" color="#1DA1F2" />
                    </a>
                    <a href="https://github.com/reymom/ic2P2ramp" target="_blank" rel="noopener noreferrer">
                        <FontAwesomeIcon icon={faGithub} size="lg" color="black" />
                    </a>
                    <a href="mailto:icramp.xyz@gmail.com" target="_blank" rel="noopener noreferrer">
                        <FontAwesomeIcon icon={faEnvelope} size="lg" color="#6f6e73" />
                    </a>
                </div>
            </div>
        </footer>
    );
};

export default Footer;