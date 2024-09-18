import { Flex, Text, Separator, ScrollArea } from "@radix-ui/themes";
import { keyframes } from "@emotion/react";
import styled from "@emotion/styled";
import "./Pricing.css";
import Header from "../../components/Header/Header";
import Calculator from "../../components/PriceCalculator/Calculator";
import PricingCard from "../../components/PricingCard";
import Footer from "../../components/Footer/Footer";
import pricingImageWebp from "../../assets/pricing/pricing-image.webp";
import pricingImageJpg from "../../assets/pricing/pricing-image-85.jpg";

const drawLine = keyframes`
  from { width: 0; }
  to { width: 100%; }
`;

const AnimatedSeparator = styled(Separator)`
  animation: ${drawLine} 1s ease-out forwards;
`;

export default function Pricing() {
  return (
    <Flex
      direction="column"
      style={{
        position: "fixed",
        height: "100%",
        width: "100%",
      }}
      className="pulsing-background"
    >
      <ScrollArea type="scroll">
        <div>
          <div className="pricing-main-container">
            <div className="pricing-image-container">
              <picture>
                <source srcSet={pricingImageWebp} type="image/webp" />
                <img
                  src={pricingImageJpg}
                  alt="pricing hero"
                  className="pricing-hero-image"
                  style={{
                    width: "100%",
                    height: "100%",
                    objectFit: "cover",
                  }}
                />
              </picture>
              <div className="pricing-gradient-overlay"></div>
            </div>
            <div className="pricing-content-container">
              <Flex className="header-container">
                <div style={{ maxWidth: "1312px", width: "100%" }}>
                  <Header px="0px" />
                </div>
              </Flex>
              <Flex className="pricing-hero-container">
                <Flex className="text-container" direction="column">
                  <Text
                    size="9"
                    weight="bold"
                    trim="both"
                    className="pricing-title"
                    mb="24px"
                  >
                    Pricing
                  </Text>
                  <AnimatedSeparator
                    size="2"
                    style={{
                      backgroundColor: "var(--cyan-4)",
                      width: "100%",
                      marginTop: "24px",
                      height: "3px",
                    }}
                  />
                  <Text
                    className="white"
                    size="5"
                    weight="medium"
                    mb="40px"
                    style={{
                      maxWidth: "542px",
                      lineHeight: "32px",
                    }}
                  >
                    Flexible pricing for every stage of your journey
                  </Text>
                  <Flex direction="row" gap="4" py="4px" align="center" mt="1">
                    <Text size="4" weight="medium" className="cyan-4">
                      Metered API
                    </Text>
                    <Separator
                      size="2"
                      orientation="vertical"
                      style={{ backgroundColor: "var(--cyan-5)" }}
                    />
                    <Text size="4" weight="medium" className="cyan-4">
                      Managed Instance
                    </Text>
                    <Separator
                      size="2"
                      orientation="vertical"
                      style={{ backgroundColor: "var(--cyan-5)" }}
                    />
                    <Text size="4" weight="medium" className="cyan-4">
                      Self-hosted
                    </Text>
                  </Flex>
                </Flex>
              </Flex>
              <Flex className="pricing-cards-container">
                <Flex
                  direction="column"
                  gap="8"
                  style={{ flex: 1, width: "100%" }}
                >
                  <Calculator />
                </Flex>
                <Flex
                  direction="column"
                  gap="9"
                  style={{ flex: 1, width: "100%" }}
                >
                  <PricingCard
                    tier="Managed Instance"
                    price="High Volume"
                    text="Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."
                    active={false}
                    enterprise={true}
                    auth={true}
                  />
                  <PricingCard
                    tier="Self-hosted"
                    price="License"
                    text="Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua."
                    active={false}
                    enterprise={true}
                    auth={true}
                  />
                </Flex>
              </Flex>
            </div>
          </div>
        </div>
        <Footer />
      </ScrollArea>
    </Flex>
  );
}
