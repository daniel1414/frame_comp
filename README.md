# Render Target Comparator

Compare your two render targets and see the difference in real time in your Rust, Vulkan-based renderer.

## Preview

https://github.com/user-attachments/assets/e841d162-0e99-42cd-9780-48e521b14fa8

## Usage

Create the Comparator object, supply the image views you want to compare and the output image view (e.g., a swapchain image, or an offscreen image, if you want to process it further). When recording your framebuffer, use the comparator's compare() function. The position of the vertical divider is in the range of (0.0; 1.0).
