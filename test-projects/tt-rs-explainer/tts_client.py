#!/usr/bin/env python3
"""
VoxCPM Voice Cloning Client (Gradio API)

Usage:
    python client.py --server http://curiosity:7860 \
                     --reference voice_sample.wav \
                     --prompt-text "The exact transcript of the reference audio" \
                     --text "The text you want the cloned voice to say" \
                     --output cloned_speech.wav
"""

import argparse
import shutil
import sys
from pathlib import Path

from gradio_client import Client, handle_file


def clone_voice(
    server_url: str,
    reference_audio_path: str,
    prompt_text: str,
    text: str,
    output_path: str,
    cfg_value: float = 2.0,
    inference_steps: int = 15,
    normalize: bool = False
) -> bool:
    """
    Send a voice cloning request to the VoxCPM Gradio API server.

    Args:
        server_url: Base URL of the VoxCPM server (e.g., http://curiosity:7860)
        reference_audio_path: Path to the reference WAV file (5-15 seconds)
        prompt_text: Exact transcript of the reference audio
        text: The text you want the cloned voice to speak
        output_path: Where to save the generated audio
        cfg_value: Classifier-free guidance value (default: 2.0)
        inference_steps: Number of inference steps (default: 15)
        normalize: Whether to normalize text (default: False)

    Returns:
        True if successful, False otherwise
    """
    # Validate reference audio exists
    ref_path = Path(reference_audio_path)
    if not ref_path.exists():
        print(f"Error: Reference audio file not found: {reference_audio_path}")
        return False

    print(f"Connecting to {server_url}...")
    print(f"  Reference audio: {ref_path.name}")
    print(f"  Prompt text: {prompt_text[:50]}{'...' if len(prompt_text) > 50 else ''}")
    print(f"  Target text: {text[:50]}{'...' if len(text) > 50 else ''}")

    try:
        client = Client(server_url)

        result = client.predict(
            text_input=text,
            prompt_wav_path_input=handle_file(str(ref_path.absolute())),
            prompt_text_input=prompt_text,
            cfg_value_input=cfg_value,
            inference_timesteps_input=inference_steps,
            do_normalize=normalize,
            api_name="/generate"
        )

        # Result is a filepath to the generated audio
        output = Path(output_path)
        output.parent.mkdir(parents=True, exist_ok=True)

        # Copy the result file to our output path
        shutil.copy(result, output)
        print(f"Success! Audio saved to: {output_path}")
        return True

    except Exception as e:
        print(f"Error: {e}")
        return False


def main():
    parser = argparse.ArgumentParser(
        description="VoxCPM Voice Cloning Client (Gradio API)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
    # Basic usage
    python client.py --server http://curiosity:7860 \\
                     --reference my_voice.wav \\
                     --prompt-text "Hello, this is a sample of my voice." \\
                     --text "Now I can say anything with my cloned voice!" \\
                     --output output.wav

    # Higher quality (slower)
    python client.py --server http://curiosity:7860 \\
                     --reference sample.wav \\
                     --prompt-text "Transcript of sample audio" \\
                     --text "Generated speech text" \\
                     --output result.wav \\
                     --steps 25
        """
    )

    parser.add_argument(
        "--server", "-s",
        default="http://curiosity:7860",
        help="VoxCPM server URL (default: http://curiosity:7860)"
    )
    parser.add_argument(
        "--reference", "-r",
        required=True,
        help="Path to reference WAV audio file (5-15 seconds)"
    )
    parser.add_argument(
        "--prompt-text", "-p",
        required=True,
        help="EXACT transcript of the reference audio"
    )
    parser.add_argument(
        "--text", "-t",
        required=True,
        help="Text you want the cloned voice to speak"
    )
    parser.add_argument(
        "--output", "-o",
        required=True,
        help="Output file path for generated audio"
    )
    parser.add_argument(
        "--cfg",
        type=float,
        default=2.0,
        help="Classifier-free guidance value (default: 2.0)"
    )
    parser.add_argument(
        "--steps",
        type=int,
        default=15,
        help="Inference steps (default: 15, higher=better quality but slower)"
    )
    parser.add_argument(
        "--normalize",
        action="store_true",
        help="Enable text normalization"
    )

    args = parser.parse_args()

    success = clone_voice(
        server_url=args.server,
        reference_audio_path=args.reference,
        prompt_text=args.prompt_text,
        text=args.text,
        output_path=args.output,
        cfg_value=args.cfg,
        inference_steps=args.steps,
        normalize=args.normalize
    )

    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
